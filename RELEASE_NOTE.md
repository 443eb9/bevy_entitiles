# What's New:

- Replaced the custom struct `*Aabb2d` with `*Rect` in Bevy.
- Switched to `avian2d` for physics.
- If `atlas` feature is enabled, add `no_flip_at`, `flip_h_at`, `flip_v_at` and `flip_both_at` for `TileLayer` , instead of adding one extra parameter.
- Renamed `TileArea` into `GridRect` .
- Use `AssetServer` for LDtk level loading.

# What's Fixed:

- Crashes when targeting WASM platforms, as `StorageBuffer` was used in versions in the past.
- Incorrect size of aabbs everywhere in this project. :( They always get shrunk by 1 unit, as I didn't know the `max` value of `IRect` is exclusive.
