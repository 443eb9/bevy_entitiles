# What's New:

- **Support multiple tilesets on one tilemap!!!**
- `material` field on tilemap bundles are now need to assign manually.
- Added `TilemapTextures`, which is an asset and replaced the original `TilemapTexture`.
- Rotating the uv of tiles is no longer supported.
- `texture_index` is renamed to `atlas_index`
- Added `texture_index` (Only when `atlas` feature enabled).
- Supported loading tilemaps that have multiple tilesets on one layer from Tiled. #22

# What's Fixed:

- Tiled objects will flicker when overlap with each other.
