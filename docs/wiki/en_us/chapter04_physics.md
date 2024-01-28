# Physics

`entitiles` includes support for [`physics_xpbd`](https://github.com/Jondolf/bevy_xpbd). The example in this chapter is in [`physics.rs`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/examples/physics.rs).

## `PhysicsTilemap`

In the example, a [`PhysicsTilemap`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/src/tilemap/physics/mod.rs#L171) is created, and you can perform operations such as filling and setting on it.

The difference is that there's a boolean variable called `concat` in `fill_rect()`, which determines whether this collision box will be a whole one.

## `DataPhysicsTilemap`

In addition, there's a [`DataPhysicsTilemap`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/src/tilemap/physics/mod.rs#L91). It's a component that defines a physics layer map. The numbers filled in represent the corresponding configuration or [`PhysicsTile`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/src/tilemap/physics/mod.rs#L74). By configuring the `tiles` parameter in `new()`, you can specify the `PhysicsTile` represented by these numbers. However, since `entitiles` uses Bevy's coordinate system, meaning the `y`-axis is reversed. If you define arrays directly like in the example, they actually need to be reversed. This operation is done internally in the `new()` function. If your data has already been reversed, please use `new_flipped()`.

In [`system.rs`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/src/tilemap/physics/systems.rs#L60), you can see the specific implementation. In simple terms, this system will connect as many Tiles with the same number in your data as possible into fewer large blocks.
