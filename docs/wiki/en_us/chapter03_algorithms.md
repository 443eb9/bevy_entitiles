# Algorithms

This section will introduce the usage of some basic algorithm modules.

## A* Pathfinding

This should be one of the most classic game algorithms. The example in this section is in [`pathfinding.rs`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/examples/pathfinding.rs).

```text
/*
 * Notice that the green circle is drawn on the origin of the tile,
 * but not the center of the tile.
 */
```

When you see this comment, if you have read the previous chapter, you should understand what's going on!

In `entitiles`, the pathfinding algorithm is executed asynchronously and bound to each Tilemap. This Tilemap must contain the [`PathTilemap`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/src/tilemap/algorithm/path.rs#L21) component. Similar to `TilemapStorage`, you can use various APIs to fill/set corresponding [`PathTile`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/src/tilemap/algorithm/path.rs#L13) to determine the cost of passing through this Tile.

After setting it up, you need to obtain the [`PathFindingQueue`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/examples/pathfinding.rs#L99C29-L99C45) resource. You can schedule pathfinding tasks, and after the algorithm is executed, an extra [`Path`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/src/algorithm/pathfinding.rs#L69) component will be added to the `requester` entity. You can use its `cur_target()` method to get the index of the next Tile and use the [`index_to_world()`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/src/tilemap/coordinates.rs#L6) function to transform it into world coordinates. It's worth noting that the returned world coordinates are based on `tile_pivot` rather than the center in the conventional sense. Similarly, there's also the [`index_to_rel()`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/src/tilemap/coordinates.rs#L33) function, which can transform the index into coordinates relative to the Tilemap origin. By the way, in this [`coordinate.rs`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/src/tilemap/coordinates.rs) file, there are many functions related to coordinates that you will find useful.

## Wave Function Collapse (WFC)

A commonly used algorithm for randomly generating maps, you can use it to generate maps similar to [`Soul Knight`](https://www.taptap.cn/app/34751) and its [`prequel`](https://www.taptap.cn/app/220156). In 3D games, you can explore a game like [`Bad North`](https://www.badnorth.com/). If you want to understand this algorithm in detail, you can watch [this video](https://www.bilibili.com/video/BV19z4y127BJ). If you're an English speaker, you can watch [this video](https://www.youtube.com/watch?v=2SuvO4Gi7uY).

First, you need a rule file, similar to [`wfc_config.ron`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/examples/wfc_config.ron), which defines the connection rules for each Tile.

If you're familiar with the WFC algorithm, you can skip this paragraph. WFC is actually like Sudoku, where the algorithm needs to randomly but with certain rules fill in the options you provide into the map. The rule is the way they connect. No one would want a map where the coast connects to the desert, so you can define rules to prevent these two options from being adjacent. Similarly, you may want the coast to connect to the ocean or another coast, so you need to set rules to allow them to connect.

Next, look at the rule file. Here's an example of the first rule:

```rust
[
    // 0
    [
        // up
        [1, 4, 2],
        // right
        [0, 1, 3, 4],
        // left
        [0, 1, 2, 5],
        // down
        [1, 2, 3],
    ],
]
```

In `entitiles`, there are many modes for wave function collapse (WFC).

### Single Tile

As the name suggests, each connected unit is a single Tile, and the example [`wfc.rs`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/examples/wfc.rs) is for this mode.

`entitiles` maps the index of the rule to the index of the options you provide and defines the connection rules as up, right, down
