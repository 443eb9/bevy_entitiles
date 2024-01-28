# What's New:

- Basic support for [Tiled](https://www.mapeditor.org/) (Experimental)
- (De)Serializing for physics layers.
- Unified the format of different tilemap layers.
- Allow mapping texture index to animation for LDtk maps.
- Split the `LdtkLevelManager` into small resources and simplified the api.
- Relaxed dependencies. #11
- Removed redundant components in tilemap bundles and renamed `tilemap_transform` to `transform`.
- Added bunch of new functions for aabbs.
- Added `TilemapAxisFlip` to allow flipping the extend direction of tilemaps.
- Removed `LdtkLevelManager` when initializing LDtk entities.

# What's Fixed:

- Isometric tilemaps have the wrong default pivot. It supposed to be `[0, 0]` but actually `[0.5, 0]`.
- Colliders for isometric tiles have the wrong position when it's parent has the pivot other than `[0, 0]`.
- Tiles of tilemaps won't despawn after the tilemaps is saved.
- Wfc module panics if fail.
- Compilation bug due to winit v0.28.7 #8
- Fix broken link in README.md #9
- Hexagonal tilemaps are not affected by `TilemapPivot`.
