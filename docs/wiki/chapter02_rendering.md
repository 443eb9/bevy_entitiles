# 渲染

想要更好地理解与利用本crate中的特性，我认为也许你需要了解一些底层原理。另外，本章不会涉及过多的GPU编程的知识，所以请放心食用。

该部分很大启发于 [`bevy_ecs_tilemap`](https://github.com/StarArawn/bevy_ecs_tilemap) ，因此，你也可以将这些知识应用于 `bevy_ecs_tilemap`。

## 区块

在 `entitiles` 中，一张大的Tilemap会被分割成小的区块。如果你玩过Minecraft，那你对此一定很熟悉，这使得视锥体剔除，与你未来会看到的区块的保存与卸载都密不可分。这个区块的大小可以在你创建 `TilemapStorage` 的时候手动指定。默认值为 `16`。在导入来自LDtk或Tiled的文件时，这个值为默认值且目前(0.4.0)无法改变。

## 网格

即Mesh，在渲染中，`entitiles` 以一个区块为单位进行渲染，也就是会对每一个区块生成对应的Mesh。那么重要的是，每一块Tile是如何被转化成Mesh的。请看 [`tilemap.wgsl`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/src/render/shaders/tilemap.wgsl)。

看到 [`第30行`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/src/render/shaders/tilemap.wgsl#L30) ，`tile_pivot` 其实就是相当于对整张Mesh的全部顶点进行了偏移，使之乘以 `tile_render_size` 时能够正确缩放。

同时也许你注意到了，对于每种类型的Tilemap，都只有`4`个顶点。对应Tile的那一小片材质也会直接被摊在这张Mesh上面。而默认的 `tile_pivot = 0` ，就是代表中心在Tile Mesh的最左下角。

```text
                   +-----+
                   |     |
                   |     | ← tile_render_size.y
                   |     |
默认 `tile_pivot` → +-----+
                      ↑
                tile_render_size.x
```

## Panic!

如果你在使用含有超过2048张Tile的材质时，你会遇到以下错误

```text
wgpu error: Validation Error

Caused by:
    In Device::create_texture
      note: label = `tilemap_texture_array`
    Dimension Z value [some value] exceeds the limit of 2048
```

**你可以开启 `atlas` 特性来避免！**

这是因为在 `entitiles` 中，每张材质会被切割成材质数组(texture_array)，其中的`z`轴，就对应你在`Tile`中填入的`texture_index`。而受限于`wgpu`，`z`值最大为`2048`。如果你对此感兴趣，那么请看[`这个文件`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/src/render/texture.rs)。

在开启 `atlas` 特性之后，每张材质就不会被切割，而是根据材质的大小等信息，计算出对应的uv坐标并直接采样。

<hr>

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
