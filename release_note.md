# What's New:

- Further support for [Tiled](https://www.mapeditor.org/).
  - Layer tint and opacities.
  - Derive macros.
- Added helper functions to make staggered tilemaps easier to create.
- `wfc_pattern` generates the wrong pattern which leads to the wrong result of WFC.

# What's Fixed:

- Hexagonal tilemaps won't have the extra displacement when flipping.
- LDtk collider generation has issues with corners in left down and right down. #12 (Wrong colliders size from LDTk level)
