# What's New:

- Manually toggle the visibility of tilemaps.
- update dev-dependencies to bevy 0.13-compatible versions #16 
- Add `SpatialBundle` as default for Tiled objects even without `spawn_sprite`.
- Tiled objects are now will be applied with an offset equal to half of the tile size to adapt to the colliders.
- Rotations to Tiled objects are no longer recommended if you want to use them more than a static object.

# What's Fixed:

- Tiled map loaded with the wrong z order.
