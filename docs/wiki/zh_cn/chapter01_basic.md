# 基本API

本章主要介绍该crate中的全部你应该知道的最少的API，如果你懒得研究 `basic.rs` 这个示例的话，请务必看完这一章。

## 各类TilemapBundle

- `TilemapBundle`

在示例中，你首先会看到的Bundle便是 [`TilemapBundle`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/src/tilemap/bundles.rs#L41) 这将会是你使用最多的一个Bundle。

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

| 成员变量           | 用途                                                                                                                                                                                                                                                                                     |
| ------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `name`             | 该Tilemap的名字，这个名字会在你使用[`TilemapSaver`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/src/serializing/map/save.rs#L37) 和 [`ChunkSaveCache`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/src/serializing/chunk/save.rs#L50) 时会被用到。一般情况下你可以默认。 |
| `tile_render_size` | 该Tilemap中，每一块Tile被**渲染**的大小(px)。你可以进入[`渲染章节`]来了解更多。                                                                                                                                                                                                          |
| `slot_size`        | 该Tilemap中，每一块Tile的**逻辑**上的大小(px)。不同于`tile_render_size`，大体来讲，该变量控制的是Tile的间距，你可以将这个值设置为小于`tile_render_size`使不同Tile间重叠，反之，也可以创造空隙。你可以进入[`渲染章节`]来了解更多。                                                        |
| `ty`               | 该Tilemap的类型，目前支持方形，等角菱形与六边形，你可以查看README上面的坐标系图来了解这三种类型的地图的区别。                                                                                                                                                                            |
| `tile_pivot`       | 该Tilemap中，每一块Tile的中心。默认为`[0, 0]`，这可以控制Tile被缩放时的延伸方向。你可以进入[`渲染章节`]来了解更多。                                                                                                                                                                      |
| `layer_opacities`  | 该Tilemap的每一层**渲染层**的不透明度。`entitiles`最多支持渲染4层。你可以向Tile中添加不限量的图层，但是只有最前面的4层会被渲染。                                                                                                                                                         |
| `storage`          | 该Tilemap的存储空间。每一块Tile会以对应的实体的类型存储，你可以使用它的一些方法来填充/设置/更新Tile。                                                                                                                                                                                    |
| `transform`        | 该Tilemap的变换信息，你可以旋转(90/180/270)/平移该地图，也可以通过改变`z_index`来更改这个地图的渲染顺序，使其在其他地图之上/下。                                                                                                                                                         |
| `texture`          | 该Tilemap的材质，个人认为这个组件比较易懂。                                                                                                                                                                                                                                              |
| `animations`       | 该Tilemap存储的所有动画，你可以使用`register`方法来注册新动画，并且把返回值分配给多个Tile使他们播放同一个动画。                                                                                                                                                                          |

除了这个Bundle之外，还有2个Bundle：
- `DataTilemapBundle`
- `PureColorTilemapBundle`

前者是用于描述一张Tilemap的最少组件，一般你可以忽略，这是在本crate内部使用的Bundle.

后者是一张没有材质与动画的Tilemap，在 [`wfc_pattern.rs`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/examples/wfc_pattern.rs) 中有被使用。
