# What's New:

- ## ***Basic*** support for [LDtk](https://ldtk.io/)!!
- Slightly optimized the wfc history system and reduced its memory usage.
- Added `UiTileMaterial` (And many other related stuff, see `ui` example). Which can be rendered as ui image in an elegant way.
- Tilemaps now support multiple layers. (this can be useful when loading maps from ldtk and calculate shadow and light in the future).
- Optimized animation.
- Renamed `TileAnimation` to `AnimatedTile`.
- Non-uniform shaped tiles are no longer supported. They are useless and increases the code complexity.
- Corrected colors for tiles. Now color will be displayed as expected.
- Individual flipping for each tile.

# What's Fixed:

- `TileFlip` not working as expected in non-uniform mode.
- Aabbs for isometric tilemaps are incorrect.
- Loader causes panic when `algorithm` feature is not enabled.

# Known Issues:

- `FilterMode` doesn't really works for `UiTilemapTexture`, as the sampler type is determined by `ImagePlugin`.
