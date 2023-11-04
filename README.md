# Bevy EntiTiles

A tilemap library for bevy.

**Notice** that the crate is still in need of optimization. So don't use this in your formal projects.

Strongly recommend that you take a look at the [`bevy_ecs_tilemap`](https://github.com/StarArawn/bevy_ecs_tilemap) library first, as it is more comprehensive and mature.

## Currently Working On

- Frustum Culling ( Optimizing )

## Future Goals

- Wave Function Collapse
- Pathfinding
- Tilemap-Link
- Runtime Mesh & Texture Baking
- Tilemap Serializing
- SSAO
- Volumetric Clouds / Fog
- Wang Tilling
- Tilemap Mask

## Show Case

Platform: 10600KF RTX3070 With 1000*1000 tiles

With Frustum Culling ( Before -> After )

<div>
	<img src="./docs/imgs/without_frustum_culling.png" width="300px"/>
	<img src="./docs/imgs/with_frustum_culling.png" width="300px"/>
</div>




| Bevy ver | EntiTiles ver |
| -------- | ------------- |
| 0.11     | 0.1.0         |

