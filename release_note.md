# What's New:

- Removed unnecessary hierarchy to get *much* **much** ***much*** more (5 times faster) better performance.
- Updated dependencies.
- Added `TilemapAabbs`.
- Frustum culling for tilemaps.
- Removed `ui` stuff. They do not deserve my extra attention.
- Broke the limitation of tile animations.
- Removed support of `bevy_rapier` as to support both physics libraries are annoying.
- Much more support for physics tilemaps. (Like collider concating)

# What's Fixed:

- Tiles and tilemaps won't update after you removed them.
- Spawning another tilemap while the program is running will cause panic. (FINALLY SOLVED!!!)
- The result of `index_to_world` is incorrect.
