# Importing Maps from LDtk

If you're not familiar with [`LDtk`](https://ldtk.io), I highly recommend checking out this fantastic map editor! However, if you don't use LDtk, you can skip this chapter. There's a lot of content in this chapter, so be prepared.

## [`ldtk.rs`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/examples/ldtk.rs)

When you enter this example, you'll first see a bunch of stuff after `App`.

First, take a look at [`LdtkLoadConfig`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/src/ldtk/resources.rs#L304) at [line 68](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/examples/ldtk.rs#L68).

```rust
pub struct LdtkLoadConfig {
    pub file_path: String,
    pub asset_path_prefix: String,
    pub filter_mode: FilterMode,
    pub z_index: i32,
    pub animation_mapper: HashMap<u32, RawTileAnimation>,
    pub ignore_unregistered_entities: bool,
    pub ignore_unregistered_entity_tags: bool,
}
```

| Member Variable                  | Purpose                                                                                                                                                                                               |
| --------------------------------| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `file_path`                      | Path to the `*.ldtk` file. Since `entitiles` directly uses `std::fs::read_to_string()` to read files, you need to add the `assets/` prefix.                                                          |
| `asset_path_prefix`              | However, images and other assets still need to be read through `AssetServer`. If your map is in `assets/ldtk/map.ldtk`, you need to set this value to `ldtk`, because resources in `*.ldtk` are represented by relative paths. So you need to add this prefix to read them correctly. |
| `filter_mode`                    | Same as `filter_mode` in `TextureDescriptor`.                                                                                                                                                          |
| `z_index`                        | Baseline `z` index. Each layer imported from the `ldtk` file will be displayed as a separate map, so this value determines their maximum `z_index`, where the first layer is `z` and the second layer is `z - 1`, and so on.                       |
| `animation_mapper`               | Maps the texture of certain specific `texture_index` Tiles in the file to corresponding animations.                                                                                                  |
| `ignore_unregistered_entities`   | Ignore unregistered entities from LDtk. If closed, encountering an unregistered entity will trigger a panic.                                                                                          |
| `ignore_unregistered_entity_tags`| Ignore unregistered LDtk entity tags. You can find them in the `Entity Settings` of any entity on the `Entities` page in LDtk's top-left corner.                                                      |

Next is [`LdtkAdditionalLayers`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/src/ldtk/resources.rs#L296) at [line 85](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/examples/ldtk.rs#L85C26-L85C47).

```rust
pub struct LdtkAdditionalLayers {
    #[cfg(feature = "algorithm")]
    pub path_layer: Option<LdtkPathLayer>,
    #[cfg(feature = "physics")]
    pub physics_layer: Option<LdtkPhysicsLayer>,
}
```

This resource depends on the `algorithm` and `physics` features. These two essentially map to IntGrids in LDtk, making them special algorithm layers/physics layers.

For `physics_layer`, it's actually a disguised `DataPhysicsTilemap`. The `parent` variable indicates which layer the generated `PhysicsTilemap` should be attached to, while `identifier` is the name of the corresponding LDtk layer.

`path_layer` is easier to understand. It's explained here.

Next is a long list of `register_ldtk_entity::<T>()`. I guess you already know what this is for. This function adds a tool to translate the names of entities in LDtk into actual components for you. It accepts the `Entity Identifier` from LDtk, which you can find in the `Project Entities` panel. `register_ldtk_entity_tag::<T>()` works similarly.

Then there's [`LdtkLevelManager`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/src/ldtk/resources.rs#L317) at [line 136](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/examples/ldtk.rs#L136). It's a tool for managing all levels, allowing you to load/unload/switch and reload files.

In [`hot_reload()`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/examples/ldtk.rs#L154), you'll see [`LdtkAssets`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/src/ldtk/resources.rs#L95). It stores all assets related to this LDtk file, including tilesets, meshes, and materials for LDtk entities. It also provides common methods for you to retrieve them. You'll see this again in the parameters of the callback function shortly.

In `events()`, some related events are showcased.

Next is the callback function I mentioned earlier, namely `player_spawn`, which is called internally within the proc-macro. It allows you to conveniently add some custom components without implementing the entire `LdtkEntity` trait yourself.

### `LdtkEntity`

Finally, let's talk about the proc-macro part, which is heavily inspired by [`bevy_ecs_ldtk`](https://github.com/Trouv/bevy_ecs_ldtk).

Let's start with `LdtkEntity`. You can skip `LdtkEnum` for now. As you might have guessed, to register an entity, you must first implement the `LdtkEntity` trait. There are many attributes on this proc-macro.

| Attribute        | Purpose                                                                                                                                                                                                                                         |
| ---------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `ldtk_default`   | Indicates variables not defined in the LDtk file, like `mp` in the example.                                                                                                                                                                    |
| `ldtk_name`      | Renames the name of the field in the proc-macro; for example, what's called `HP` in LDtk can be renamed to `hp`.                                                                                                                               |
| `spawn_sprite`   | Generates a sprite for this entity. You can choose not to generate, like some Area types in the example. However, to demonstrate more supported [`TileRenderMode`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/src/ldtk/sprite.rs#L55), all are displayed. This corresponds to the options on the right side of `Editor Visual` in `Entity Settings`. |
| `global_entity`  | Indicates that this entity doesn't belong to any layer; it exists independently and won't be destroyed with its layer.                                                                                                                                                                                      |
| `callback`       | Specifies the callback function mentioned earlier.                                                                                                                                                                                              |

**Note: If you need to use custom enum types, you must use the corresponding wrapper.**

The reason for this is that Rust doesn't allow implementing a trait from another crate for a struct from a different crate. Here we need to implement `Into<T> for FieldInstance` (`T` represents `Option<YourEnum> or Option<Vec<YourEnum>>`), where `FieldInstance` is the object that stores the values defined in LDtk, and during deserialization, it needs to call `<FieldInstance as Into<T>>::into()`. However, it's not allowed to implement `Into<T> for FieldInstance` here. So we define a wrapper here and implement `Into<Wrapper> for FieldInstance`.

### `LdtkEnum`

This is relatively simple, and the reason for the Wrapper has been explained. There are only two understandable attributes: `ldtk_name` and `wrapper_derive`.

### `LdtkEntityTag`

It's also straightforward, marking the component for the corresponding Tag. Don't forget to `register_ldtk_entity_tag::<T>()`!

## [`ldtk_wfc.rs`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/examples/ldtk_wfc.rs)

This example shows how to use LDtk maps for wfc.

This only describes the steps different from regular wfc. If you need a tutorial on regular wfc, please refer to chapter 3.

Actually, there's only one line; when defining `WfcSource`, choose `WfcSource::LdtkMapPattern()`. This makes `LdtkLevelManager` automatically load all levels as options for wfc. Here, you can choose to apply the results to one map or multiple maps.

The remaining code only works if you choose `LdtkWfcMode::MultiMaps`, which essentially checks which level the small square has moved to and makes `LdtkLevelManager` load the corresponding level.

