# ✨Bevy EntiTiles 🎈

A tilemap library for bevy. With many algorithms built in.

This crate is still in need of **optimization and development**. ~~So don't use this in your formal projects.~~ Anyway, I think it's already capable to be a part of your project.

Strongly recommend that you take a look at the [`bevy_ecs_tilemap`](https://github.com/StarArawn/bevy_ecs_tilemap) library first, as it is more comprehensive and mature.

This repo is under maintenance as long as this message exists. ( Hope this message can bring you peace of mind. Yeah, so childish. )

Notice that the following progress may be not up to date. Check the `README.md` in `dev` brach to get the newest progress!

## Currently Working On

- Migrate to 0.12

## Future Goals

- Wave Function Collapse ( Release & Optimization )
- Pathfinding ( Optimization )
- Tilemap-Link
- Runtime Mesh & Texture Baking
- Tilemap Serializing
- SSAO
- Volumetric Clouds / Fog
- Wang Tilling
- Tilemap Mask
- Frustum Culling ( Isometric Specific Optimization )

## Show Case

Platform: 10600KF With 1000*1000 tiles

### Frustum Culling

Before -> After

<div>
	<img src="./docs/imgs/without_frustum_culling.png" width="300px"/>
	<img src="./docs/imgs/with_frustum_culling.png" width="300px"/>
</div>

### Wave Function Collapse

In the following case, each tile has at least one corresponding color gap with its neighboring tiles

<div>
	<img src="./docs/imgs/wfc.png" width="500px">
</div>

Due to the lack of optimization and the straightforward implementation of this algorithm, its execution time increases rapidly as the map size grows. And the likelihood of failure also increases.

| Size  | Time(avg of 3 tests) |
| ----- | -------------------- |
| 5x5   | 3.92 ms              |
| 10x10 | 33.312 ms            |
| 20x20 | 490.950 ms           |
| 30x30 | 2280.121 s           |
| 50x50 | 18838.542 s          |

*Bevy 0.11.3, crate 0.2.0, None weighted, `retrace_strength` = 2*

## Versions

| Bevy ver | EntiTiles ver |
| -------- | ------------- |
| 0.12.x   | Working On    |
| 0.11.x   | 0.1.x         |

