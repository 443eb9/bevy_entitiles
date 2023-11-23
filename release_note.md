# What's New:
- Use texture atlas instead of texture array. Which provides the possibility to make non-uniform shaped tiles.

# What's Fixed:
- The color of tilemap textures do not really affects the real color.
- Tilemaps cover the other objects. (`FloatOrd`)
- `FillArea` will cause panic when the destination is at the edge.