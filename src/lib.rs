//! Open Waters Charts: the nautical-style maps from <https://openwaters.io/charts/> as basemaps.
//!
//! Open Waters publishes each map as a MapLibre style that carries its base map, bathymetry
//! and symbology together, so the plugin adds that one style through RustGIS's `map.addTiles`
//! (which needs the `network` permission), the way the Basemaps plugin adds OpenFreeMap.
//! Choosing a map replaces the web basemap already there; `{"add": true}` stacks it instead.

use rustgis_plugin::{Kind, Request, Value, host, json};

/// `plugin.json`, for its translations.
const MANIFEST: &str = include_str!("../plugin.json");

/// An Open Waters map.
pub struct Chart {
    /// The command id.
    pub id: &'static str,
    /// The id kept on the tile layer, to find it again.
    pub layer_id: &'static str,
    pub name: &'static str,
    pub style_url: &'static str,
}

pub const CHARTS: &[Chart] = &[
    Chart { id: "seamap", layer_id: "openwaters-seamap", name: "Seamap", style_url: "https://tiles.openwaters.io/seamap/style.json" },
    Chart {
        id: "seascape",
        layer_id: "openwaters-seascape",
        name: "Seascape",
        style_url: "https://tiles.openwaters.io/seascape/style.json",
    },
    // Seascape's other views of the same data. They take none of the chart's options.
    Chart {
        id: "seascape-relief",
        layer_id: "openwaters-seascape-relief",
        name: "Seascape Relief",
        style_url: "https://tiles.openwaters.io/seascape/style.json?view=relief",
    },
    Chart {
        id: "seascape-hillshade",
        layer_id: "openwaters-seascape-hillshade",
        name: "Seascape Hillshade",
        style_url: "https://tiles.openwaters.io/seascape/style.json?view=hillshade",
    },
    Chart {
        id: "seascape-topo",
        layer_id: "openwaters-seascape-topo",
        name: "Seascape Topo",
        style_url: "https://tiles.openwaters.io/seascape/style.json?view=topo",
    },
];

fn handle(req: Request) -> Result<Value, String> {
    if let (Kind::Command, Some(chart)) = (req.kind, CHARTS.iter().find(|c| c.id == req.id)) {
        return add(&req, chart);
    }
    match (req.kind, req.id.as_str()) {
        (Kind::Command, "remove") => remove(&req),
        (_, other) => Err(format!("Open Waters Charts has no '{other}'")),
    }
}

rustgis_plugin::export!(handle);

/// The style's address, with Seascape's options (unit, safety depth, shading) as its query.
pub fn style_url(chart: &Chart, req: &Request) -> Result<String, String> {
    let mut query = Vec::new();
    if chart.id == "seascape" {
        if let Some(unit) = req.str("unit") {
            if !["m", "ft", "fm"].contains(&unit) {
                return Err(format!("unit must be m, ft or fm, not '{unit}'"));
            }
            query.push(format!("unit={unit}"));
        }
        if let Some(safety) = req.f64("safety") {
            if safety < 0.0 {
                return Err(format!("safety must be a depth in metres, not {safety}"));
            }
            query.push(format!("safety={safety}"));
        }
        if let Some(shading) = req.str("shading") {
            if !["relief", "bands"].contains(&shading) {
                return Err(format!("shading must be relief or bands, not '{shading}'"));
            }
            query.push(format!("shading={shading}"));
        }
    }
    Ok(if query.is_empty() { chart.style_url.to_string() } else { format!("{}?{}", chart.style_url, query.join("&")) })
}

/// The parameters of `map.addTiles` for a chart. No attribution of its own: each style carries
/// the credits Open Waters asks for ("© Open Waters: Seamap/Seascape", OpenStreetMap, …).
pub fn add_params(chart: &Chart, url: &str, replace: bool) -> Value {
    json!({
        "url": url, "kind": "style", "name": format!("Open Waters {}", chart.name),
        "maxZoom": 14, "overlay": false, "id": chart.layer_id, "replace": replace,
    })
}

fn add(req: &Request, chart: &Chart) -> Result<Value, String> {
    let url = style_url(chart, req)?;
    let out = host::run("map.addTiles", add_params(chart, &url, !req.bool("add").unwrap_or(false)))?;
    let name = req.tr(MANIFEST, chart.name, &[]);
    host::notify(&req.tr(MANIFEST, "Basemap: {name} (not for navigation)", &[("name", &name)]));
    Ok(out)
}

/// Removes every Open Waters map on the map.
fn remove(req: &Request) -> Result<Value, String> {
    let tiles = host::run("map.tiles", json!({}))?;
    let mut removed = Vec::new();
    for chart in CHARTS {
        if tiles.as_array().is_some_and(|t| t.iter().any(|t| t["id"] == chart.layer_id)) {
            host::run("map.removeTiles", json!({"id": chart.layer_id}))?;
            removed.push(req.tr(MANIFEST, chart.name, &[]));
        }
    }
    host::notify(&if removed.is_empty() {
        req.tr(MANIFEST, "No Open Waters maps on the map", &[])
    } else {
        req.tr(MANIFEST, "Removed {names}", &[("names", &removed.join(", "))])
    });
    Ok(json!({"removed": removed}))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustgis_plugin::serde_json;

    fn request(id: &str, params: Value) -> Request {
        Request {
            kind: Kind::Command,
            id: id.to_string(),
            params,
            layers: Default::default(),
            files: Default::default(),
            lang: "en".to_string(),
            panel: Default::default(),
            panel_changed: Vec::new(),
        }
    }

    #[test]
    fn manifest_matches_handlers() {
        let manifest: Value = serde_json::from_str(MANIFEST).unwrap();
        let commands: Vec<&str> = manifest["commands"].as_array().unwrap().iter().filter_map(|c| c["id"].as_str()).collect();
        assert_eq!(commands.len(), CHARTS.len() + 1);
        for chart in CHARTS {
            assert!(commands.contains(&chart.id), "{} isn't in plugin.json", chart.id);
        }
        assert!(commands.contains(&"remove"));
        for chart in CHARTS {
            assert!(chart.style_url.starts_with("https://tiles.openwaters.io/"), "{}", chart.id);
        }
        assert_eq!(manifest["permissions"], json!(["edit", "network"]));
    }

    #[test]
    fn builds_style_urls() {
        let seascape = &CHARTS[1];
        assert_eq!(style_url(seascape, &request("seascape", json!({}))).unwrap(), seascape.style_url);
        let url = style_url(seascape, &request("seascape", json!({"unit": "ft", "safety": 3, "shading": "bands"}))).unwrap();
        assert_eq!(url, "https://tiles.openwaters.io/seascape/style.json?unit=ft&safety=3&shading=bands");
        assert!(style_url(seascape, &request("seascape", json!({"unit": "km"}))).is_err());
        // Seamap and Seascape's views have no options.
        assert_eq!(style_url(&CHARTS[0], &request("seamap", json!({"unit": "ft"}))).unwrap(), CHARTS[0].style_url);
        let topo = CHARTS.iter().find(|c| c.id == "seascape-topo").unwrap();
        assert_eq!(style_url(topo, &request("seascape-topo", json!({"unit": "ft"}))).unwrap(), topo.style_url);
    }

    #[test]
    fn builds_add_tiles_params() {
        let p = add_params(&CHARTS[1], CHARTS[1].style_url, true);
        assert_eq!(p["replace"], true);
        assert_eq!(p["kind"], "style");
        assert_eq!(p["id"], "openwaters-seascape");
    }
}
