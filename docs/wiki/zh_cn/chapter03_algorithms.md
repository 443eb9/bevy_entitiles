# 算法

这里会介绍一些基本算法模块的使用。

## A*寻路

应该算是最经典的游戏算法之一了吧。本节示例为 [`pathfinding.rs`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/examples/pathfinding.rs)。

```text
/*
 * Notice that the green circle is drew on the origin of the tile,
 * but not the center of the tile.
 */
```

在看到这个注释的时候，如果你已经阅读完了上一章，那你一定能明白是怎么回事了吧！

在 `entitiles` 中，寻路算法是异步执行的，并且绑定到每一张Tilemap上。这张Tilemap必须含有 [`PathTilemap`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/src/tilemap/algorithm/path.rs#L21) 这个组件。与 `TilemapStorage` 相似，你也可以使用各种API来填充/设置对应的 [`PathTile`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/src/tilemap/algorithm/path.rs#L13) 来决定走过这块Tile的开销。

设置完后，你需要再获取 [`PathFindingQueue`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/examples/pathfinding.rs#L99C29-L99C45) 这个Resource。你可以计划寻路任务，在算法执行完毕后，在 `requester` 这个实体上便会多出一个 [`Path`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/src/algorithm/pathfinding.rs#L69) 组件。你可以使用它的 `cur_target()` 方法获取下一块Tile的索引，并使用 [`index_to_world()`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/src/tilemap/coordinates.rs#L6) 函数将其变换为世界坐标。值得注意的是，返回的世界坐标位于 `tile_pivot` 而非那个一般意义上的中心。同样还有 [`index_to_rel()`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/src/tilemap/coordinates.rs#L33) 函数，可以将索引变换为相对于Tilemap原点的坐标。顺便提一句，在这个 [`coordinate.rs`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/src/tilemap/coordinates.rs) 中，有许多关于坐标的函数，你一定能用得上。

## 波函数坍缩

一个常用于随机生成地图的算法，你可以使用它生成类似于 [`元气骑士`](https://www.taptap.cn/app/34751) 及其 [`前传`](https://www.taptap.cn/app/220156) 的地图。在3D游戏中，你可以了解一下 [`Bad North`](https://www.badnorth.com/) 这款游戏。如果你想详细了解这个算法，那么你可以观看 [`这个视频`](https://www.bilibili.com/video/BV19z4y127BJ) 。如果你是英语用户，那你可以观看 [`这个视频`](https://www.youtube.com/watch?v=2SuvO4Gi7uY)。

首先你需要一个规则文件，就像[`wfc_config.ron`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/examples/wfc_config.ron) 一样，它定义了每一块Tile的连接规则。

如果你了解wfc算法，请跳过本段。wfc实际上就像是数独，算法需要将你给的的选项随机但是有一定规则地填入地图中。这个规则就是连接的方式。没人会希望他的地图里会出现海岸连接着沙漠这种沙雕情况，所有你可以定义规则，使这两个选项不相邻。同样，你也许希望海岸连接着海洋，或是另一块海岸，那你也要制定规则允许它们相连。

接下来看向规则文件。这里以第一条规则为例

```rust
[
    // 0
    [
        // up
        [1, 4, 2],
        // right
        [0, 1, 3, 4],
        // left
        [0, 1, 2, 5],
        // down
        [1, 2, 3],
    ],
]
```

在 `entitiles` 中，波函数坍缩（以下简称wfc）有很多种模式

### 单Tile

顾名思义，每一个连接的单元都是单块Tile，[`wfc.rs`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/examples/wfc.rs) 这个示例就是这个模式。

`entitiles` 会将规则的索引映射为你给定的选项的索引，并且在连接规则中，按照：上右下左的方式定义。也就是说，选项`0`的上面只能是 `1, 4, 2` 中的其中一个。在你读入规则的时候，`entitiles` 会对每一条规则审查，避免出现冲突。

### 地图图案

也就是将上述的单块Tile变成了一块可以包含 颜色层/算法层/[`物理层`](./chapter04_physics.md) 的地图图案。

### 多层地图图案

多张地图层叠在一起，没什么好解释的。

### LDtk相关

这个模式在这里暂时不具体介绍，总之就是将LDtk的地图作为选项进行wfc，你可以查看 [`导入来自LDtk的地图`](./chapter06_ldtk_import.md) 来了解更多。

让我们回到示例(`wfc.rs`，其他示例同样适用)。在`41`行，我们读取了规则，并指定了地图的类型。随后，`WfcSource` 即为你给定的选项。此处的`from_texture_indices()` 意为直接将你提供的规则文件中的每一条规则都转化为对应的Tile。比如像上面这条规则，就相当于 `texture_index = 0` 的Tile上面可以连接 `texture_index = 1 or 4 or 2` 的Tile。其他属性不可自定义。当然你也可以手动指定。

接下来的 `WfcRunner` 即为启动该算法的核心。你可以不指定权重(`with_weights()`)或使用你自定义的采样器(`with_custom_sampler()`)来对每一次坍缩的结果进行选择，直接默认所有选项的权重相等。还有一些其他的设置。

- `with_retrace_settings()` wfc算法并不是每次都可以成功的，失败时它会选择回退，那么这个方法可以指定它的回退强度和最大回退次数。在`0.4.0`版本下，不建议使用较高的强度，一般`1`即可。
- `with_history_settings()` 既然要回退，那么历史记录就是必须的了。你可以选择缩小最多保存的历史记录（默认为 `100`） 来减少内存使用，不过一般也不需要调整。注意，如果这个值过小可能会导致回退失败。

最后，向同一个实体再插入一个空白的 `TilemapBundle`，剩下的就可以交给 `entitiles` 了。
