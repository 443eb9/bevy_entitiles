# Bevy EntiTiles 🗺️

A 2d tilemap library for bevy. With many useful algorithms/tools built in.

*Warning: Before using this crate, be prepared for potentially AnNoYiNg API changes that come with almost each version update.*

This repo is under maintenance as long as this message exists. ( Hope this message can bring you peace of mind. Yeah, that's childish :p )

It's **NOT** recommended to use the code in `dev` branch! There's full of incomplete code and even errors! But `master` branch would be ok if you can't wait to try out new features.

## Future Goals

*The higher the priority, the more towards the front in the following list.*

- Pathfinding (for LDtk maps)
- Wave Function Collapse ( Collapse map from LDtk; Optimization )
- Chunk Unloading
- Infinite Tilemap
- Runtime Mesh & Texture Baking
- Wang Tilling
- Tilemap Mask
- [Tiled](https://www.mapeditor.org/) Support
- Volumetric Clouds / Fog
- SSAO
- Lighting
- Realtime Shadow
- ~~Frustum Culling~~
- ~~Physics~~
- ~~[LDtk](https://ldtk.io/) Full Support~~

## Known Issues

- The success probability of the wfc algorithm significantly decreased after switching to `LookupHeap`. ( And that's why I didn't switch to `LookupHeap` )
- Spawning another tilemap while the program is running will cause panic.

## Feature Flags

| Flag             | Funtionality                                                                                         |
| ---------------- | ---------------------------------------------------------------------------------------------------- |
| `algorithm`      | Implementation of algorithms                                                                         |
| `debug`          | Show some debug info including aabbs for chunks and tilemaps, path finding results etc.              |
| `ldtk`           | LDtk support.                                                                                        |
| `physics_rapier` | Physics support for [`bevy_rapier`](https://github.com/dimforge/bevy_rapier)                         |
| `physics_xpbd`   | Physics support for [`bevy_xpbd`](https://github.com/Jondolf/bevy_xpbd), like setting colliders etc. |
| `serializing`    | Save and load the tilemap from files. Also contains tools for upgrading files.                       |
| `ui`             | Support renderring tiles as ui image.                                                                |

## Coordinate Systems

The x and y axes in the tilemaps are the index axes. And those x and y on a single tile means the actual mesh size. Which you can control using `tile_render_size`.

<div>
	<img src="./docs/imgs/coordinate_systems.jpg" width="500px">
</div>

*`legs` here are mathematically incorrect, please consider it as a new concept.*

## Show Cases & Performance

Platform: 10600KF

### Frustum Culling

<div>
	<img src="./docs/imgs/without_frustum_culling.png" width="300px"/>
	<img src="./docs/imgs/with_frustum_culling.png" width="300px"/>
</div>

> *Bevy 0.11.3, crate 0.1.1, 1000x1000 tiles*

### Animation

<div>
	<img src="./docs/imgs/animation.gif" width="500px">
</div>

> *Bevy 0.12.1, crate 0.2.3*

### Ui System

<div>
	<img src="./docs/imgs/ui.gif" width="500px">
</div>

> *Bevy 0.12.1, crate 0.2.4*

### Pathfinding

The pathfinding algorithm is very fast.

Notice this tests are done with **synchronized pathfinding**. Which means whole algorithm will figure the path out in one frame. But since `0.2.1`, the asynchronized one in implemented. So the algorithm can complete a part of the pathfinding and continue it in the next frame. This will make it even smoother.

<div>
	<img src="./docs/imgs/pathfinding.png" width="500px">
</div>

| Size      | Time(avg of 3 tests) ms |
| --------- | ----------------------- |
| 100x100   | 12.00                   |
| 500x500   | 295.67                  |
| 1000x1000 | 1384.33                 |

> *Bevy 0.12, crate 0.2.1, using `pathfinding` example*

### Wave Function Collapse

In the following case, each tile has at least one corresponding color gap with its neighboring tiles.

<div>
	<img src="./docs/imgs/wfc.gif" width="500px">
</div>

| Size    | Time(avg of 3 tests) ms | Time(avg of 3 tests) ms |
| ------- | ----------------------- | ----------------------- |
| 10x10   | 33.312                  | 16.264 (3)              |
| 20x20   | 490.950                 | 96.009 (3)              |
| 30x30   | 2,280.121               | 335.697 (6)             |
| 50x50   | 18,838.542              | 2,095.428 (8)           |
| 100x100 | (Not measurable)        | 32,309.045 (16)         |

> *Column 1: Bevy 0.11.3, crate 0.2.0, NoneWeighted; Column 2: Bevy 0.12, crate 0.2.1, NoneWeighted, `max_retrace_factor` = number in parentheses*

## Limitations

- Supports up to 4 rendered layers* in one tilemap.
- Supports up to 64 animations in one tilemap. (this will no longer exists in the future)
- Supports up to 16 length of animation sequences.

*\* Rendered layer means the layer that will be **rendered**. You can insert as much layers as you want, but only the 4 top layers will be rendered.*

## Special Thanks & References

- SSAO & Volumetric Clouds / Fog inspired by [this video](https://www.bilibili.com/video/BV1KG411U7uk/).
- [`bevy_ecs_tilemap`](https://github.com/StarArawn/bevy_ecs_tilemap) I took this crate as the reference and learnt the basis of bevy rendering!
- [`bevy_ecs_ldtk`](https://github.com/Trouv/bevy_ecs_ldtk) I was confused and have no idea about how to instantiate LDtk entities before I read this crate! I also learnt proc macros and many other things from this crate!

## Assets

- [SunnyLand_by_Ansimuz-extended.png](https://ansimuz.itch.io/sunny-land-pixel-game-art)

## Versions

| Bevy ver | EntiTiles ver | LDtk ver      |
| -------- | ------------- | ------------- |
| 0.12.x   | 0.2.3-0.2.5   | 1.4.1         |
| 0.12.x   | 0.2.0-0.2.2   | Not supported |
| 0.11.x   | 0.1.x         | Not supported |
