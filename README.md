# Bevy EntiTiles 🗺️

A 2d tilemap library for bevy. With many useful algorithms/tools built in.

Try to be the most **comprehensive**, **performant**, and **up-to-date** 2d tilemap crate for bevy.

This repo is under maintenance as long as this message exists!!

It's **NOT** recommended to use the code in `dev` branch! There's full of incomplete code and even errors! But `master` branch would be ok if you can't wait to try out new features.

This crate is largely inspired from [`bevy_ecs_tilemap`](https://github.com/StarArawn/bevy_ecs_tilemap)(Rendering) and [`bevy_ecs_ldtk`](https://github.com/Trouv/bevy_ecs_ldtk)(LDtk entity spawning)

## Future Goals

*The higher the priority, the more towards the front in the following list.*

- [Tiled](https://www.mapeditor.org/) Support
- Runtime Mesh & Texture Baking
- Custom Material
- Volumetric Clouds / Fog
- SSAO
- Realtime Lighting
- Wang Tiling
- Tilemap Mask
- ~~Frustum Culling~~
- ~~Pathfinding~~
- ~~Physics~~
- ~~[LDtk](https://ldtk.io/) Support~~
- ~~Infinite Tilemap~~
- ~~Chunk Unloading~~
- ~~Tilemap Serializing~~

## Feature Flags

| Flag          | Funtionality                                                                            |
| ------------- | --------------------------------------------------------------------------------------- |
| `algorithm`   | Implementation of algorithms                                                            |
| `atlas`       | Use calculated uv coordinates on a entire texture instead of using texture arrays.      |
| `debug`       | Show some debug info including aabbs for chunks and tilemaps, path finding results etc. |
| `ldtk`        | [LDtk](https://ldtk.io/) support.                                                       |
| `physics`     | Physics support using [`bevy_xpbd`](https://github.com/Jondolf/bevy_xpbd).              |
| `serializing` | Save and load the tilemap from files. Also contains tools for upgrading files.          |
| `tiled`       | [Tiled](https://www.mapeditor.org/) support.                                            |

## Coordinate Systems

The x and y axes in the tilemaps are the index axes. And those x and y on a single tile means the actual mesh size. Which you can control using `tile_render_size`.

<div>
	<img src="https://raw.githubusercontent.com/443eb9/bevy_entitiles/master/docs/imgs/coordinate_systems.jpg" width="500px">
</div>

*`legs` here are mathematically incorrect, please consider it as a new concept.*

## Show Cases & Performance

Platform: 10600KF

**Notice: Due to the performance overhead caused by the recorder, the fps value maybe inaccurate!**

### LDtk

The gif on the right is the map generated with wave function collapse. And the orange boxes in the left image is procedural generated colliders.

<div>
	<img src="https://raw.githubusercontent.com/443eb9/bevy_entitiles/master/docs/imgs/ldtk.png" width="250px">
	<img src="https://raw.githubusercontent.com/443eb9/bevy_entitiles/master/docs/imgs/ldtk_wfc.gif" width="250px">
</div>

> *Bevy 0.12.1, crate 0.2.6, LDtk 1.4.1*

### Chunk Unloading

I know you are confused about these weird boxes, so please check the [`chunk_unloading`](https://github.com/443eb9/bevy_entitiles/blob/master/examples/chunk_unloading.rs) example if you want to get further info.

<div>
	<img src="https://raw.githubusercontent.com/443eb9/bevy_entitiles/master/docs/imgs/chunk_unloading.gif" width="500px">
</div>

> *Bevy 0.12.1, crate 0.2.7*

### Pathfinding

Notice this tests are done with **synchronized pathfinding**. Which means whole algorithm will figure the path out in one frame. But since `0.2.1`, the asynchronized one in implemented. So the algorithm can complete a part of the pathfinding and continue it in the next frame. This will make it even smoother.

**Notice: The synchronized pathfinding was removed in 0.3.0.**

<div>
	<img src="https://raw.githubusercontent.com/443eb9/bevy_entitiles/master/docs/imgs/pathfinding.png" width="500px">
</div>

| Size      | Time(avg of 3 tests) ms | Time(avg of 3 tests) ms |
| --------- | ----------------------- | ----------------------- |
| 100x100   | 12.00                   | 10.85                   |
| 500x500   | 295.67                  | 191.24                  |
| 1000x1000 | 1384.33                 | 993.75                  |

> *Column 1: Bevy 0.12.0, crate 0.2.1, using `pathfinding` example</br>
> Column 2: Bevy 0.12.0, crate 0.3.0*

### Wave Function Collapse

In the following case, each tile has at least one corresponding color gap with its neighboring tiles.

**Notice: The funtionality of visualizing the process of wfc was removed in 0.2.6**

<div>
	<img src="https://raw.githubusercontent.com/443eb9/bevy_entitiles/master/docs/imgs/wfc.png" width="500px">
</div>

| Size    | Time(avg of 3 tests) ms | Time(avg of 3 tests) ms | Time(avg of 3 tests) ms |
| ------- | ----------------------- | ----------------------- | ----------------------- |
| 10x10   | 33.312                  | 16.264 (3)              | 0.516(8)                |
| 20x20   | 490.950                 | 96.009 (3)              | 3.344(8)                |
| 30x30   | 2,280.121               | 335.697 (6)             | 12.280(8)               |
| 50x50   | 18,838.542              | 2,095.428 (8)           | 75.143(8)               |
| 100x100 | (Not measurable)        | 32,309.045 (16)         | 999.414(8)              |

> *Column 1: Bevy 0.11.3, crate 0.2.0, NoneWeighted</br>
> Column 2: Bevy 0.12, crate 0.2.1, NoneWeighted, `max_retrace_factor` = number in parentheses</br>
> Column3: Bevy 0.12.1, crate 0.2.6, NoneWeighted, `max_retrace_factor` = number in parentheses*

## Limitations

- Supports up to 4 rendered layers* in one tilemap.

*\* Rendered layer means the layer that will be **rendered**. You can insert as much layers as you want, but only the 4 top layers will be rendered.*

## References

- SSAO & Volumetric Clouds / Fog inspired by [this video](https://www.bilibili.com/video/BV1KG411U7uk/).

## Assets

- [SunnyLand_by_Ansimuz-extended.png](https://ansimuz.itch.io/sunny-land-pixel-game-art)
- [Cavernas_by_Adam_Saltsman.png](https://adamatomic.itch.io/cavernas)

## Versions

*LDtk version is the version that json api has changed. So you can also use 1.5.2 in 0.2.7.</br>See [this](https://ldtk.io/json/next/#changes) for more information.*

| Bevy ver | EntiTiles ver | LDtk ver      | Tiled ver     |
| -------- | ------------- | ------------- | ------------- |
| 0.12.x   | 0.4.0         | 1.5.3         | 1.10.2        |
| 0.12.x   | 0.3.0         | 1.5.3         | Not supported |
| 0.12.x   | 0.2.7         | 1.5.1         | Not supported |
| 0.12.x   | 0.2.3-0.2.6   | 1.4.1         | Not supported |
| 0.12.x   | 0.2.0-0.2.2   | Not supported | Not supported |
| 0.11.x   | 0.1.x         | Not supported | Not supported |

*Versions before 0.3.0 are not named following [`Semantic Versioning`](https://semver.org/)*
