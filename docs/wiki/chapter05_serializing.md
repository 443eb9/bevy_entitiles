# 保存与读取

对于你创建的Tilemap，你可以直接使用本crate内部的组件进行相对较高性能的保存，而不是使用反射。

## [`save_and_load.rs`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/examples/save_and_load.rs)

在这个示例中，包含对整张Tilemap的保存与读取。重点在 [`save_and_load()`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/examples/save_and_load.rs#L112) 函数中。你可以使用 [`TilemapSaver`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/src/serializing/map/save.rs#L37) 与 [`TilemapLoader`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/src/serializing/map/load.rs#L38) 进行保存和读取。

```rust
pub struct TilemapSaver {
    /// For example if path = C:\\maps, then the crate will create:
    /// ```
    /// C
    /// └── maps
    ///     └── (your tilemap's name)
    ///         ├── tilemap.ron
    ///         └── (and other data)
    /// ```
    ///
    /// If the mode is `TilemapSaverMode::MapPattern`, then the crate will create:
    /// ```
    /// C
    /// └── maps
    ///     └── (your tilemap's name).pattern
    /// ```
    pub path: String,
    pub mode: TilemapSaverMode,
    pub layers: TilemapLayer,
    pub texture_path: Option<String>,
    pub remove_after_save: bool,
}
```

- `path` 见注释
- `mode` 你可以选择将这张地图保存为地图图案的形式，以便用于wfc。具体示例为 [`wfc_pattern.rs`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/examples/wfc_pattern.rs)。
- `layers` 一张Tilemap被分为`3`个层：颜色层/算法层/物理层。对应`3`种Tilemap [`TilemapBundle`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/src/tilemap/bundles.rs#L41), [`PathTilemap`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/src/tilemap/algorithm/path.rs#L21), [`PhysicsTilemap`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/src/tilemap/physics/mod.rs#L171)。你可以有选择地保存。
- `texture_path` 我们当然不可能直接保存材质，而是保存其路径，例如 `test_square.png`。当然如果你的Tilemap没有材质，这个值就不需要填写。
- `remove_after_save` 字面意思，开启后crate会在保存完毕之后删除所有与这张Tilemap相关的实体。

```rust
pub struct TilemapLoader {
    /// For example if the file tree look like:
    ///
    /// ```
    /// C
    /// └── maps
    ///     └── beautiful map
    ///         ├── tilemap.ron
    ///         └── (and other data)
    /// ```
    /// Then path = `C:\\maps` and map_name = `beautiful map`
    pub path: String,
    pub map_name: String,
    pub layers: TilemapLayer,
}
```

- `path` 见注释
- `map_name` 见注释
- `layers` 同 `TilemapSaver`

## [`chunk_unloading`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/examples/chunk_unloading.rs)

实际上这应该是某几个功能的细化实现。包含对区块的加载卸载，以及对相机出入区块的检测。

先看前者。

不同于对于地图的保存/读取，对区块的保存是使用的 `Resource` 的方式。因为对于一张地图，非常可能出现几个区块同时需要保存，而区块实际上是抽象概念并没有对应实体，因此使用 `Resource` 模式更好。

首先你需要插入配置文件。在 [63行](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/examples/chunk_unloading.rs#L63) 与 [67行](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/examples/chunk_unloading.rs#L67) 中，分别插入了保存与读取的配置，当然你也可以根据你自己的情况只插入其中一个。由于保存/读取区块非常消耗性能，因此这个工作会被分摊到每一帧。通过修改 `chunks_per_frame` ，你可以调整每帧保存/读取的区块数。不过不是很推荐提升这个值，一般 `1` 就够用了。

我猜你已经注意到了其他两个资源：

- `FrustumCulling` 启用/关闭视锥体剔除
- `CameraAabbScale` 对相机的 `Aabb` 进行缩放

因为我们这里需要观察到区块的加载与卸载，因此需要关闭视锥体剔除。否则你就只能看到你视野内的区块了。而缩放 `Aabb` 也是同样的道理。默认的 `Aabb` 是和你看到的内容一样大的，就会导致无法观察。这两个资源都是调试使用的。不建议你在正式发布时带着这两个资源。

接下来的位于 `115` 行的 [`reserve_many()`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/examples/chunk_unloading.rs#L115) ，相当于你对crate说：

> 嘿！这些区块是存在的！但是也许它们现在不在地图上。总之我的摄像机靠过去你一定要告诉我！

“告诉”的具体体现便是发出对应的Event。可以看到在 [`on_update`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/examples/chunk_unloading.rs#L173) 中，我们对事件进行了监听，并且做出对应操作。在这里我们直接读取硬盘中的文件。你也可以实时生成对应的区块。

<hr>

# Saving and Loading

For the Tilemaps you create, you can directly use internal components of this crate for relatively high-performance saving instead of using reflection.

## [`save_and_load.rs`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/examples/save_and_load.rs)

In this example, saving and loading of entire Tilemaps are included. The focus is on the [`save_and_load()`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/examples/save_and_load.rs#L112) function. You can use [`TilemapSaver`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/src/serializing/map/save.rs#L37) and [`TilemapLoader`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/src/serializing/map/load.rs#L38) for saving and loading.

```rust
pub struct TilemapSaver {
    /// For example if path = C:\\maps, then the crate will create:
    /// ```
    /// C
    /// └── maps
    ///     └── (your tilemap's name)
    ///         ├── tilemap.ron
    ///         └── (and other data)
    /// ```
    ///
    /// If the mode is `TilemapSaverMode::MapPattern`, then the crate will create:
    /// ```
    /// C
    /// └── maps
    ///     └── (your tilemap's name).pattern
    /// ```
    pub path: String,
    pub mode: TilemapSaverMode,
    pub layers: TilemapLayer,
    pub texture_path: Option<String>,
    pub remove_after_save: bool,
}
```

- `path`: See comments.
- `mode`: You can choose to save this map as a map pattern for use in WFC. A specific example is [`wfc_pattern.rs`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/examples/wfc_pattern.rs).
- `layers`: A Tilemap is divided into `3` layers: color layer, algorithm layer, and physics layer. Corresponding to `3` types of Tilemaps: [`TilemapBundle`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/src/tilemap/bundles.rs#L41), [`PathTilemap`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/src/tilemap/algorithm/path.rs#L21), [`PhysicsTilemap`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/src/tilemap/physics/mod.rs#L171). You can choose which to save.
- `texture_path`: We don't directly save textures, just their paths, for example, `test_square.png`. If your Tilemap doesn't have textures, this value doesn't need to be filled.
- `remove_after_save`: Literally, after saving, the crate will delete all entities related to this Tilemap if this is enabled.

```rust
pub struct TilemapLoader {
    /// For example if the file tree look like:
    ///
    /// ```
    /// C
    /// └── maps
    ///     └── beautiful map
    ///         ├── tilemap.ron
    ///         └── (and other data)
    /// ```
    /// Then path = `C:\\maps` and map_name = `beautiful map`
    pub path: String,
    pub map_name: String,
    pub layers: TilemapLayer,
}
```

- `path`: See comments.
- `map_name`: See comments.
- `layers`: Same as `TilemapSaver`.

## [`chunk_unloading`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/examples/chunk_unloading.rs)

Actually, this should be a refined implementation of some features. It includes loading and unloading of chunks, as well as detection of camera entry and exit from chunks.

Let's start with the former.

Unlike saving/loading maps, saving/loading chunks uses the resource method. This is because for a map, several chunks may need to be saved simultaneously, and chunks are actually abstract concepts without corresponding entities, so using the resource mode is better.

First, you need to insert the configuration file. In [line 63](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/examples/chunk_unloading.rs#L63) and [line 67](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/examples/chunk_unloading.rs#L67), the configurations for saving and loading are inserted, respectively, but you can insert only one according to your own situation. Since saving/loading chunks consumes a lot of performance, this work will be spread out over each frame. By modifying `chunks_per_frame`, you can adjust the number of chunks saved/loaded per frame. However, it's not recommended to increase this value too much; generally, `1` is enough.

You may have noticed two other resources:

- `FrustumCulling`: Enable/disable frustum culling.
- `CameraAabbScale`: Scale the `Aabb` of the camera.

Since we need to observe the loading and unloading of chunks here, frustum culling needs to be turned off. Otherwise, you will only see the chunks within your field of view. Scaling `Aabb` is the same reason. The default `Aabb` is the same size as what you see, which makes observation impossible. These two resources are for debugging purposes. It's not recommended to bring them along when officially released.

Next is [`reserve_many()`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/examples/chunk_unloading.rs#L115) at line 115, which is like telling the crate:

> Hey! These chunks exist! But maybe they are not on the map right now. Anyway, when my camera approaches, you must tell me!

The specific manifestation of "telling" is to emit the corresponding Event. As you can see in [`on_update`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/examples/chunk_unloading.rs#L173), we listen for events and perform corresponding operations. Here

, we directly read the files from the disk. You can also generate corresponding chunks in real time.
