# What's New:

- (De)Serializing for physics layers.
- Unified the format of different tilemap layers.
- Allow mapping texture index to animation for LDtk maps.
- Split the `LdtkLevelManager` into small resources and simplified the api.

# What's Fixed:

- Isometric tilemaps have the wrong default pivot. It supposed to be `[0, 0]` but actually `[0.5, 0]`.
- Colliders for isometric tiles have the wrong position when it's parent has the pivot other than `[0, 0]`.
- Tiles of tilemaps won't despawn after the tilemaps is saved.
- Wfc module panics if wfc is failed.
- Program sometimes will panic ( Source buffer/texture is missing the `COPY_SRC` usage flag ) right after a tilemap is spawned.
