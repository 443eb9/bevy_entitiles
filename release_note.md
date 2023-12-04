# What's New:

- Slightly optimized the wfc history system and reduced its memory usage.
- Added tilemap editor.
- Added `UiTileMaterial` (And many other related stuff, see `ui` example). Which can be rendered as ui image in an elegant way.

# What's Fixed:

- `TileFlip` not working as expected in non-uniform mode.
- Aabbs for isometric tilemaps are incorrect.

# Known Issues:

- `FilterMode` doesn't really works for `UiTilemapTexture`, as the sampler type is determined by `ImagePlugin`. I don't know how to use custom sampler in `AsBindGroup`. :(

# What's Next: