# What's New:
- Added `physics` feature back.
- Moved linux-related dependencies to `dev-dependencies`.

# What's Fixed:

- Materials of tilemaps that are added to the scene at runtime will not be loaded correctly.
- Entity(LDtk)/Object(Tiled) sprites are not rendered.
- An unreachable pattern in `wfc.rs` when `ldtk` feature is disabled.
- Switching between tilemaps from Tiled causes panic.
