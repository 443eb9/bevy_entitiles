# What's New:
- Use texture atlas instead of texture array.
- Added non-uniform shaped tiles thanks to the first "What's New".
- Added tile anchor.

# What's Fixed:
- The color of tilemap textures do not really affects the real color.
- Tilemaps cover the other objects. (`FloatOrd`)
- `FillArea` will cause panic when the destination is at the edge.

# What's Next:
- Wave Function Collapse support for map chunks. Comparing to collapse single tiles, manually create chunks and collapse using them seems more common.
- So the tilemap serilization will be implemented as well.