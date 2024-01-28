# 物理

`entitiles` 包含对 [`physics_xpbd`](https://github.com/Jondolf/bevy_xpbd) 的支持。本章示例为 [`physics.rs`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/examples/physics.rs)。

## `PhysicsTilemap`

在示例中，创建了一个 [`PhysicsTilemap`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/src/tilemap/physics/mod.rs#L171) 同样的你也可以进行一填充/设置等操作。

不同的是，在 `fill_rect()` 中有一个 `concat` 的布尔类型变量，它决定了这个碰撞箱会不会是一整个。

## `DataPhysicsTilemap`

此外，还有一个 [`DataPhysicsTilemap`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/src/tilemap/physics/mod.rs#L91) 。它是定义一张物理层地图的组件。其中填充的数字就是对应的配置，或者说对应的 [`PhysicsTile`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/src/tilemap/physics/mod.rs#L74) 。通过配置 `new()` 中的 `tiles` 参数，你可以指定这些数字代表的 `PhysicsTile` 。不过由于 `entitiles` 使用的是 `bevy` 的坐标系，也就是说 `y` 轴是相反的。如果你是像示例中这样直接定义数组的，那么实际上是需要反转的。`new()` 函数内部完成了这个操作。如果你的数据已经经过翻转，那么请使用 `new_flipped()`。

在 [`system.rs`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/src/tilemap/physics/systems.rs#L60) 中，你可以看到具体的实现。简单来说，这个系统会把你给的数据中所有的数字相同的Tile尽可能地连成数量少的大块。

<hr>

# Physics

`entitiles` includes support for [`physics_xpbd`](https://github.com/Jondolf/bevy_xpbd). The example in this chapter is in [`physics.rs`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/examples/physics.rs).

## `PhysicsTilemap`

In the example, a [`PhysicsTilemap`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/src/tilemap/physics/mod.rs#L171) is created, and you can perform operations such as filling and setting on it.

The difference is that there's a boolean variable called `concat` in `fill_rect()`, which determines whether this collision box will be a whole one.

## `DataPhysicsTilemap`

In addition, there's a [`DataPhysicsTilemap`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/src/tilemap/physics/mod.rs#L91). It's a component that defines a physics layer map. The numbers filled in represent the corresponding configuration or [`PhysicsTile`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/src/tilemap/physics/mod.rs#L74). By configuring the `tiles` parameter in `new()`, you can specify the `PhysicsTile` represented by these numbers. However, since `entitiles` uses Bevy's coordinate system, meaning the `y`-axis is reversed. If you define arrays directly like in the example, they actually need to be reversed. This operation is done internally in the `new()` function. If your data has already been reversed, please use `new_flipped()`.

In [`system.rs`](https://github.com/443eb9/bevy_entitiles/blob/0.4.0/src/tilemap/physics/systems.rs#L60), you can see the specific implementation. In simple terms, this system will connect as many Tiles with the same number in your data as possible into fewer large blocks.
