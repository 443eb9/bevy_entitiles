# What's New:

- (De)Serializing for physics layers.
- Unified the format of different tilemap layers.

# What's Fixed:

- Isometric tilemaps have the wrong default pivot. It supposed to be `[0, 0]` but actually `[0.5, 0]`.
- Colliders for isometric tiles have the wrong position when it's parent has the pivot other than `[0, 0]`.
- Tiles of tilemaps won't despawn after the tilemaps is saved.
