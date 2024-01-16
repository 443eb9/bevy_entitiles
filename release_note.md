# What's New:

- Removed unnecessary hierarchy to get *much* **much** ***much*** more (5 times faster) better performance.
- Relaxed requirements of dependency versions.
- Added `TilemapAabbs`.
- Frustum culling for tilemaps.
- Removed `ui` stuff. They do not deserve my extra attention.
- Broke the limitation of tile animations.
- Removed support of `bevy_rapier` as to support both physics libraries is tiring.
- Much further support for physics tilemaps.
- Pathfinding rework. Using bevy async compute to asynchronize the process.
- Use bevy async compute to asynchronize wfc.
- Supported `LdtkEnum`s to become the entity tags.
- Supported atlas mode for those extremely large ( more than 2048 atlases ) textures.

# What's Fixed:

- Tiles and tilemaps won't update after you removed them.
- Spawning another tilemap while the program is running will cause panic. (FINALLY SOLVED!!!)
- The result of `index_to_world` is incorrect.
- Deriving `LdtkEntity` and `LdtkEnum` requires to `use` them first.
