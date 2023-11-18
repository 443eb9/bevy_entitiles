# Bevy EntiTiles 🗺️

A tilemap library for bevy. With many algorithms built in.

Strongly recommend that you take a look at the [`bevy_ecs_tilemap`](https://github.com/StarArawn/bevy_ecs_tilemap) library first. I took this crate as the reference and learnt the basis of bevy rendering.

This repo is under maintenance as long as this message exists. ( Hope this message can bring you peace of mind. Yeah, that's childish :p )

Notice that the following progress may **not up to date**. Check the `README.md` in `dev` brach to get the latest progress!

## Currently Working On

- SSAO & Volumetric Clouds / Fog

## Future Goals

- Wave Function Collapse ( Optimization )
- ~~Pathfinding~~
- Tilemap-Link
- Runtime Mesh & Texture Baking
- Tilemap Serializing
- Chunk Unloading
- SSAO
- Volumetric Clouds / Fog
- Wang Tilling
- Tilemap Mask
- Frustum Culling ( Isometric Specific Optimization )
- ~~Physics~~

## Known Issues Currently Unresolved

- The success probability of the wfc algorithm significantly decreased after switching to `LookupHeap`. ( And that's the reason why I didn't switch to `LookupHeap` )

## Performance

Platform: 10600KF

### Frustum Culling

<div>
	<img src="./docs/imgs/without_frustum_culling.png" width="300px"/>
	<img src="./docs/imgs/with_frustum_culling.png" width="300px"/>
</div>

> *Bevy 0.11.3, crate 0.1.1, 1000x1000 tiles*

### Pathfinding

The pathfinding algorithm is very fast.

Notice this tests are done with **synchronized pathfinding**. Which means whole algorithm will figure the path out in one frame. But since `0.2.1`, we the supports asynchronized one. The algorithm can complete a part of the pathfinding and continue it in the next frame. This will make it even smoother.

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

In the following case, each tile has at least one corresponding color gap with its neighboring tiles

<div>
	<video src="./docs/vids/wfc.mp4" controls="controls" width="500"></video>
</div>

| Size    | Time(avg of 3 tests) ms | Time(avg of 3 tests) ms |
| ------- | ----------------------- | ----------------------- |
| 10x10   | 33.312                  | 16.264 (3)              |
| 20x20   | 490.950                 | 96.009 (3)              |
| 30x30   | 2,280.121               | 335.697 (6)             |
| 50x50   | 18,838.542              | 2,095.428 (8)           |
| 100x100 | (Not measurable)        | 32,309.045 (16)         |

> *Column 1: Bevy 0.11.3, crate 0.2.0, NoneWeighted; Column 2: Bevy 0.12, crate 0.2.1, NoneWeighted, `max_retrace_factor` = number in parentheses*

## Versions

| Bevy ver | EntiTiles ver |
| -------- | ------------- |
| 0.12.x   | 0.2.x         |
| 0.11.x   | 0.1.x         |
