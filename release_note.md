# What's New:

- Further support for [Tiled](https://www.mapeditor.org/).
  - Layer tint and opacities supported.
  - Added `shape_as_collider` macro attribute for `TiledObject`.
- Added helper functions to make staggered tilemaps easier to create.

# What's Fixed:

- Hexagonal tilemaps won't have the extra displacement when flipping.
- LDtk collider generation have issues with corners in left down and right down. #12 (Wrong colliders size from LDTk level)
