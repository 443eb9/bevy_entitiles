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
