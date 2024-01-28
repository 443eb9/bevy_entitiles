# Rendering

To better understand and utilize the features of this crate, I believe you may need to grasp some underlying principles. Additionally, this chapter will not delve too deeply into GPU programming knowledge, so feel free to dive in.

This section is heavily inspired by [`bevy_ecs_tilemap`](https://github.com/StarArawn/bevy_ecs_tilemap), so you can also apply this knowledge to `bevy_ecs_tilemap`.

## Chunk

In `entitiles`, a large Tilemap is divided into small chunks. If you've played Minecraft, you'll be familiar with this concept, as it enables view frustum culling and the saving and unloading of chunks you will encounter in the future. The size of these chunks can be manually specified when you create `TilemapStorage`. The default value is `16`. When importing files from LDtk or Tiled, this value is set to the default and currently (0.4.0) cannot be changed.

## Grid

Meshes, in rendering, `entitiles` renders per chunk, which means a Mesh is generated for each chunk. The crucial point here is how each Tile is transformed into a Mesh. Please refer to [`tilemap.wgsl`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/src/render/shaders/tilemap.wgsl).

At [`line 30`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/src/render/shaders/tilemap.wgsl#L30), `tile_pivot` actually offsets all vertices of the entire Mesh, allowing them to be correctly scaled when multiplied by `tile_render_size`.

You may also notice that for each type of Tilemap, there are only `4` vertices. The corresponding small piece of texture for the Tile will be directly mapped onto this Mesh. The default `tile_pivot = 0`, meaning the center is at the bottom left corner of the Tile Mesh.

```text
                       +-----+
                       |     |
                       |     | ← tile_render_size.y
                       |     |
Default `tile_pivot` → +-----+
                          ↑
                  tile_render_size.x
```

## Panic!

If you encounter the following error when using a texture containing more than 2048 Tiles:

```text
wgpu error: Validation Error

Caused by:
    In Device::create_texture
      note: label = `tilemap_texture_array`
    Dimension Z value [some value] exceeds the limit of 2048
```

**You can enable the `atlas` feature to avoid this!**

This is because in `entitiles`, each texture is sliced into a texture array, where the `z` axis corresponds to the `texture_index` you fill in the `Tile`. Limited by `wgpu`, the maximum value for `z` is `2048`. If you're interested, please refer to [this file](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/src/render/texture.rs).

After enabling the `atlas` feature, each texture will not be sliced but sampled directly based on information such as the size of the texture and UV coordinates calculation.
