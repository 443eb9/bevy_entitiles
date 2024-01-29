# Basic API

This chapter primarily introduces all the minimal APIs you should know in this crate. If you are not inclined to explore the `basic.rs` example, make sure to go through this chapter.

## Various TilemapBundles

- `TilemapBundle`

In the example, the first bundle you will encounter is [`TilemapBundle`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/src/tilemap/bundles.rs#L41), which will be the most frequently used bundle.

```rust
pub struct TilemapBundle {
    pub name: TilemapName,
    pub tile_render_size: TileRenderSize,
    pub slot_size: TilemapSlotSize,
    pub ty: TilemapType,
    pub tile_pivot: TilePivot,
    pub layer_opacities: TilemapLayerOpacities,
    pub storage: TilemapStorage,
    pub transform: TilemapTransform,
    pub axis_flip: TilemapAxisFlip,
    pub texture: TilemapTexture,
    pub animations: TilemapAnimations,
}
```

| Member Variable    | Purpose                                                                                                                                                                                                                                                                                                                                 |
| ------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `name`             | The name of the Tilemap, which will be used when you use [`TilemapSaver`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/src/serializing/map/save.rs#L37) and [`ChunkSaveCache`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/src/serializing/chunk/save.rs#L50). Usually, you can leave this as default.                   |
| `tile_render_size` | The size (in pixels) at which each Tile is **rendered** in the Tilemap. You can explore the [`Rendering Section`](./chapter02_rendering.md) for more details.                                                                                                                                                                           |
| `slot_size`        | The logical size (in pixels) of each Tile in the Tilemap. Unlike `tile_render_size`, this variable generally controls the spacing of Tiles. You can set this to be smaller than `tile_render_size` to overlap Tiles or vice versa to create gaps. You can explore the [`Rendering Section`](./chapter02_rendering.md) for more details. |
| `ty`               | The type of the Tilemap, currently supporting square, isometric rhombus, and hexagonal types. You can refer to the coordinate system diagram in the README to understand the differences between these three types of maps.                                                                                                             |
| `tile_pivot`       | The center of each Tile in the Tilemap. Defaults to `[0, 0]`, which controls the direction of extension when scaling Tiles. You can explore the [`Rendering Section`](./chapter02_rendering.md) for more details.                                                                                                                       |
| `layer_opacities`  | The opacity of each **render layer** in the Tilemap. `entitiles` supports rendering up to 4 layers. You can add an unlimited number of layers to Tiles, but only the front 4 layers will be rendered.                                                                                                                                   |
| `storage`          | The storage space of the Tilemap. Each Tile is stored according to the type of the corresponding entity, and you can use some methods to fill/set/update Tiles.                                                                                                                                                                         |
| `transform`        | The transformation information of the Tilemap. You can rotate (90/180/270)/translate the map, or change the `z_index` to change the rendering order of the map, making it appear above/below other maps.                                                                                                                                |
| `texture`          | The texture of the Tilemap, which is a straightforward component.                                                                                                                                                                                                                                                                       |
| `animations`       | All animations stored in the Tilemap. You can use the `register` method to register new animations and assign the return value to multiple Tiles to play the same animation.                                                                                                                                                            |

Apart from this bundle, there are 2 more bundles:
- `DataTilemapBundle`
- `PureColorTilemapBundle`

The former is used to describe the minimum components of a Tilemap, which you can generally ignore as it is used internally in this crate.

The latter is a Tilemap without textures and animations, which is used in [`wfc_pattern.rs`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/examples/wfc_pattern.rs).
