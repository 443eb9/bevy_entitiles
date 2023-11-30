# What's New:
- Use texture atlas instead of texture array.
- Added non-uniform shaped tiles thanks to the first "What's New".
- Added tile anchor.
- Added tilemap serialization & deserialization.
- Migrated to bevy 0.12.1

# What's Fixed:
- The color of tilemap textures do not really affects the real color.
- Pure color tilemap does not really support.
- Tilemaps cover the other objects. (`FloatOrd`)
- `FillArea` will cause panic when the destination is at the edge.

# Known Issues:
- Program panics if load a tilemap twice.

# What's Next:
- Wave Function Collapse support for map patterns. Comparing to collapse single tiles, manually create chunks and collapse using them seems more common.