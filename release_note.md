# What's New:

- Added README for examples.
- Custom materials for tilemaps. Implement `TilemapMaterial` and add the handle to the tilemap to access the your data when rendering.
- Migrated to bevy 0.13.
- Use cached tilemap data instead of extracting every frame for rendering. FPS++ -> 215fps (10600KF + 3070 + `stress_test` example)

# What's Fixed:

- LDtk wfc ignores the background.
- Multi layer pattern wfc uses the wrong data for `tile_render_size` and `tile_slot_size`.
