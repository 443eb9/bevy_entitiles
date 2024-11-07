# Bevy EntiTiles 🗺️

A 2d tilemap library for bevy. With many useful algorithms/tools built in.

It's **NOT** recommended to use the code in the `dev` branch! There's full of incomplete code and even errors in it! But the `master` branch would be ok if you can't wait to try out new features.

Currently, documentation is not very comprehensive. If you encounter any problems, please feel free to open an issue or simply ping me or DM *@443eb9* on the bevy discord server.

This crate is largely inspired from [`bevy_ecs_tilemap`](https://github.com/StarArawn/bevy_ecs_tilemap)(Rendering) and [`bevy_ecs_ldtk`](https://github.com/Trouv/bevy_ecs_ldtk)(LDtk entity spawning)

## Why EntiTiles

- **Up-to-date** Once the new version of bevy is released, `bevy_entitiles` will catch up *mostly* in 12 hours.
- **Performant** Able to render 1000x1000 tilemaps at 200+fps on 10600KF + 3070.
- **Various Built-in Stuff** Useful algorithms(pathfinding, WFC...) and tools(LDtk and Tiled importer...) are built in.

## Warning!

You should **NEVER** install the following versions! They contain critical bugs and are not recommended to use!

**0.6.0**

## Limitations

- Supports up to 4 rendered layers* per tilemap.

*\* Rendered layer means the layer that will be **rendered**. You can insert as many layers as you want, but only the 4 top layers will be rendered.*

## Future Goals

*The higher the priority, the more towards the front in the following list.*

- More Tilemap Shapes (Triangle, Voronoi)
- Pathfinding (Parallel A* Pathfinding, Jump Point Search)
- [LDtk](https://ldtk.io/) Support (Automatically map `EntityRef` to real entities)
- [Tiled](https://www.mapeditor.org/) Support (Algorithm stuff, auto map `object` type to real entities)
- Tilemap Serializing (Custom Binary Format)
- Wang Tiling
- Tilemap Mask
- ~~Frustum Culling~~
- ~~Physics~~
- ~~Infinite Tilemap~~
- ~~Chunk Unloading~~
- ~~Custom Material~~

*Looking for render features that have been removed? They're moved into [`bevy_incandescent`](https://github.com/443eb9/bevy_incandescent) (a 2d lighting crate currently wip)!*

## Feature Flags

| Flag             | Funtionality                                                                            |
| ---------------- | --------------------------------------------------------------------------------------- |
| `algorithm`      | Implementation of algorithms                                                            |
| `atlas`          | Use calculated uv coordinates on a entire texture instead of using texture arrays.      |
| `debug`          | Show some debug info including aabbs for chunks and tilemaps, path finding results etc. |
| `ldtk`           | [LDtk](https://ldtk.io/) support.                                                       |
| `multi-threaded` | Support algorithms to run asynchronously. Disable this if you are targeting wasm.       |
| `physics`        | Physics support using [`avian`](https://github.com/Jondolf/avian).              |
| `serializing`    | Save and load the tilemap from files. Also contains tools for upgrading files.          |
| `tiled`          | [Tiled](https://www.mapeditor.org/) support.                                            |

## Coordinate Systems

The x and y axes in the tilemaps are the index axes. And those x and y on a single tile mean the actual mesh size. Which you can control using `tile_render_size`.

<div>
	<img src="https://raw.githubusercontent.com/443eb9/bevy_entitiles/master/docs/imgs/coordinate_systems.jpg" width="500px">
</div>

*`legs` here are mathematically incorrect, please consider it as a new concept.*

## Showcases

*See the `README` in `examples`*

## Assets

- [SunnyLand_by_Ansimuz-extended.png](https://ansimuz.itch.io/sunny-land-pixel-game-art)
- [Cavernas_by_Adam_Saltsman.png](https://adamatomic.itch.io/cavernas)

## Versions

*LDtk version is the version that json api has changed. So you can also use 1.5.2 in 0.2.7. See [this](https://ldtk.io/json/next/#changes) for more information.*

*It doesn't mean all the features from the corresponding versions of LDtk and Tiled are supported, but this crate is using the source tilemap file generated from them!*

| Bevy ver | EntiTiles ver             | LDtk ver      | Tiled ver     | WASM Support |
| -------- | ------------------------- | ------------- | ------------- | ------------ |
| 0.14.x   | 0.12.0 (Haven't Released) | 1.5.3         | 1.11.0        | Yes          |
| 0.14.x   | 0.10.0-0.11.0             | 1.5.3         | 1.11.0        | No           |
| 0.13.x   | 0.6.0-0.9.0               | 1.5.3         | 1.10.2        | No           |
| 0.12.x   | 0.4.0-0.5.0               | 1.5.3         | 1.10.2        | No           |
| 0.12.x   | 0.3.0                     | 1.5.3         | Not supported | No           |
| 0.12.x   | 0.2.7                     | 1.5.1         | Not supported | No           |
| 0.12.x   | 0.2.3-0.2.6               | 1.4.1         | Not supported | No           |
| 0.12.x   | 0.2.0-0.2.2               | Not supported | Not supported | No           |
| 0.11.x   | 0.1.x                     | Not supported | Not supported | No           |

*Versions before 0.3.0 are not named following [`Semantic Versioning`](https://semver.org/)*
