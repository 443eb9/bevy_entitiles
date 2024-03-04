# What's New:

- Added `TilemapBaker` which allows you to bake the tilemap into a single texture at runtime.
- Manually toggle the visibility of tilemaps.
- update dev-dependencies to bevy 0.13-compatible versions #16 
- Add `SpatialBundle` as default for Tiled objects even without `spawn_sprite`.
- Tiled objects are now will be applied with an offset equal to half of the tile size to adapt to the colliders.
- Rotations to Tiled objects are no longer recommended if you want to use them more than a static object.
- Some render code cleanup.
- Tilemap z indices are now using `f32` instead of `i32`.
- Pathfinding rework. Use `PathTilemaps` instead.
- Single threaded algorithm implementations for special cases. Disable `multi-threaded` feature to use them.
- Transform `TileFlip` into bitflags, and make fields in `TileLayer` public.
- Some helper functions for `TileLayer` to help create a layer neater.

# What's Fixed:

- Tiled map loaded with the wrong z order.
- `algorithm` module cannot be compiled on wasm (Error compiling for webassembly #17)
- The message of tilemap pipeline error is annoying.
