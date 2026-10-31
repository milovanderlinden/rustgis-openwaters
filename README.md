# Open Waters Charts

[Open Waters'](https://openwaters.io/charts/) nautical-style maps as basemaps for RustGIS:

- **[Seamap](https://openwaters.io/charts/seamap/)**: depth shading, contours and soundings,
  seamarks (buoys, beacons, lights and harbours), light sectors, traffic separation schemes,
  rocks and obstructions, and offshore infrastructure.
- **[Seascape](https://openwaters.io/charts/seascape/)**: the seafloor, from a mosaic of global
  open bathymetry, with depth shading, contours, soundings and drying areas.

**Not for navigation.** These maps look like nautical charts but aren't: seamarks are
crowd-sourced from OpenStreetMap, and depths are approximate and not reduced to a chart datum.
Always consult official charts.

## Commands

On the **Open Waters** tab:

- **Seamap**, and under **Seascape** the **Seascape Chart**: use that map as the basemap,
  replacing the current web basemap. With `{"add": true}` it stacks over the current basemaps
  instead.
- **Seascape Relief**, **Seascape Hillshade** and **Seascape Topo**: Seascape's other views of
  the same data, as basemaps in the same way. They draw shaded relief from elevation tiles, which
  needs a RustGIS that draws `hillshade` and `color-relief` layers; older versions show them
  nearly blank.
- **Remove Charts**: take the Open Waters maps off the map.

Seascape takes the options of its style:

| Param | Values | Default |
|---|---|---|
| `unit` | `m`, `ft`, `fm`: the depth unit | `m` |
| `safety` | a depth in metres: shallower water is shaded as a hazard | `2` |
| `shading` | `relief` (continuous) or `bands` (depth areas, as on ENCs) | `relief` |

```sh
rustgis-cli run --plugin dist/openwaters --sample \
  --cmd 'plugin:openwaters/seascape={"unit": "ft", "shading": "bands"}' \
  --cmd 'share.exportMap={"path": "seascape.png"}'
```

## Terms of use

The tiles, styles and sprites at `tiles.openwaters.io` are
[CC BY 4.0](https://creativecommons.org/licenses/by/4.0/), credited as
"© Open Waters: Seamap" and "© Open Waters: Seascape". They derive from OpenStreetMap data
(© OpenStreetMap contributors, ODbL) and many bathymetry sources, each with its own terms.
Each style carries its credits, which RustGIS shows in the corner of the map and in exports.

## Permissions

edit and network (adding web tile layers, which the map loads from Open Waters' servers)

## Build

```sh
rustup target add wasm32-unknown-unknown
python scripts/package.py
```

This writes `dist/openwaters/` (a folder to load with **Plugins › Plugin Manager › Settings ›
Add Plugin Folder…**) and `dist/openwaters-0.1.0.rgplugin`.
