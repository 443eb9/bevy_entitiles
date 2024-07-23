use avian2d::prelude::{Collider, Friction, RigidBody};
use bevy::{
    app::{App, Plugin, Update},
    ecs::{component::Component, entity::Entity, event::Event, system::Commands},
    math::{IRect, IVec2, UVec2, Vec2},
    reflect::Reflect,
    utils::HashMap,
};

use crate::{
    math::TileArea,
    tilemap::{
        buffers::{PackedPhysicsTileBuffer, PhysicsTileBuffer, Tiles},
        chunking::storage::{
            ChunkedStorage, EntityChunkedStorage, PackedPhysicsTileChunkedStorage,
        },
    },
};

pub mod systems;

pub struct EntiTilesPhysicsTilemapPlugin;

impl Plugin for EntiTilesPhysicsTilemapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                systems::spawn_colliders,
                systems::data_physics_tilemap_analyzer,
            ),
        );

        app.register_type::<PhysicsTileSpawn>()
            .register_type::<PhysicsTilemap>()
            .register_type::<DataPhysicsTilemap>()
            .register_type::<PhysicsTile>();

        app.add_event::<PhysicsTileSpawn>();
    }
}

/// An event that is fired when a physics tile is spawned after the analysis
/// of a data tilemap.
///
/// Which means if the physics tile is not from a `DataPhysicsTilemap`, this event
/// won't be fired.
#[derive(Event, Debug, Clone, Copy, Reflect)]
pub struct PhysicsTileSpawn {
    pub tile: Entity,
    pub tilemap: Entity,
    /// The possible integer representation of the tile in the corresponding `DataPhysicsTile`.
    pub int_repr: Option<i32>,
}

/// Possible representations of a serialized physics tilemap.
#[cfg(feature = "serializing")]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Reflect)]
pub enum SerializablePhysicsSource {
    Data(DataPhysicsTilemap),
    Buffer(super::buffers::PackedPhysicsTileBuffer),
}

/// All the vertices of a physics collider.
#[derive(Debug, Clone, Reflect)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub enum PhysicsCollider {
    Convex(Vec<Vec2>),
    Polyline(Vec<Vec2>),
}

impl PhysicsCollider {
    pub fn as_verts(&self) -> &Vec<Vec2> {
        match self {
            PhysicsCollider::Convex(verts) => verts,
            PhysicsCollider::Polyline(verts) => verts,
        }
    }

    pub fn as_verts_mut(&mut self) -> &mut Vec<Vec2> {
        match self {
            PhysicsCollider::Convex(verts) => verts,
            PhysicsCollider::Polyline(verts) => verts,
        }
    }
}

#[derive(Debug, Clone, Reflect)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub struct PackedPhysicsTile {
    pub parent: IVec2,
    pub collider: PhysicsCollider,
    pub physics_tile: PhysicsTile,
}

impl Into<PhysicsTile> for PackedPhysicsTile {
    fn into(self) -> PhysicsTile {
        self.physics_tile
    }
}

impl Tiles for PackedPhysicsTile {}

impl PackedPhysicsTile {
    pub fn spawn(&self, commands: &mut Commands) -> Entity {
        let mut entity = commands.spawn(match self.collider.clone() {
            PhysicsCollider::Convex(verts) => Collider::convex_hull(verts).unwrap(),
            PhysicsCollider::Polyline(verts) => Collider::polyline(verts, None),
        });
        if self.physics_tile.rigid_body {
            entity.insert(RigidBody::Static);
        }
        if let Some(friction) = &self.physics_tile.friction {
            entity.insert(Friction::new(*friction));
        }
        entity.id()
    }
}

#[derive(Debug, Clone, Reflect)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub struct PhysicsTile {
    pub rigid_body: bool,
    pub friction: Option<f32>,
}

impl Default for PhysicsTile {
    fn default() -> Self {
        Self {
            rigid_body: true,
            friction: Default::default(),
        }
    }
}

impl Tiles for PhysicsTile {}

/// This can used to spawn a optimized physics tilemap.
///
/// Once the component is added, the crate will figure out the least amount of colliders
/// needed to represent the tilemap and spawn them.
#[derive(Component, Debug, Clone, Reflect)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub struct DataPhysicsTilemap {
    pub(crate) origin: IVec2,
    pub(crate) data: Vec<i32>,
    pub(crate) size: UVec2,
    pub(crate) air: i32,
    pub(crate) tiles: HashMap<i32, PhysicsTile>,
}

impl DataPhysicsTilemap {
    /// Create a new physics tilemap from a data array.
    ///
    /// As the y axis in array and bevy is flipped, this method will flip the array.
    /// If your data is already flipped, use `new_flipped` instead.
    pub fn new(
        origin: IVec2,
        data: Vec<i32>,
        size: UVec2,
        air: i32,
        tiles: HashMap<i32, PhysicsTile>,
    ) -> Self {
        assert_eq!(
            data.len(),
            size.x as usize * size.y as usize,
            "Data size mismatch!"
        );

        let mut flipped = Vec::with_capacity(data.len());
        for y in 0..size.y {
            for x in 0..size.x {
                flipped.push(data[(x + (size.y - y - 1) * size.x) as usize]);
            }
        }

        DataPhysicsTilemap {
            origin,
            data: flipped,
            size,
            air,
            tiles,
        }
    }

    /// Create a new physics tilemap from a data array. Without flipping the array.
    pub fn new_flipped(
        origin: IVec2,
        flipped_data: Vec<i32>,
        size: UVec2,
        air: i32,
        tiles: HashMap<i32, PhysicsTile>,
    ) -> Self {
        assert_eq!(
            flipped_data.len(),
            size.x as usize * size.y as usize,
            "Data size mismatch!"
        );

        DataPhysicsTilemap {
            origin,
            data: flipped_data,
            size,
            air,
            tiles,
        }
    }

    /// Try to get the tile at the given index.
    ///
    /// This will return the air tile if the index is out of bounds.
    #[inline]
    pub fn get_or_air(&self, index: UVec2) -> i32 {
        if index.x >= self.size.x || index.y >= self.size.y {
            self.air
        } else {
            self.data[(index.x + index.y * self.size.x) as usize]
        }
    }

    /// Map the tile id to a physics tile.
    #[inline]
    pub fn get_tile(&self, value: i32) -> Option<PhysicsTile> {
        self.tiles.get(&value).cloned()
    }

    #[inline]
    pub fn set(&mut self, index: UVec2, value: i32) {
        self.data[(index.x + index.y * self.size.x) as usize] = value;
    }
}

/// A tilemap with physics tiles.
#[derive(Component, Debug, Clone, Reflect)]
pub struct PhysicsTilemap {
    pub(crate) storage: EntityChunkedStorage,
    pub(crate) spawn_queue: Vec<(IRect, PhysicsTile, Option<i32>)>,
    pub(crate) data: PackedPhysicsTileChunkedStorage,
}

impl PhysicsTilemap {
    /// Create a new physics tilemap with default chunk size.
    ///
    /// Use `new_with_chunk_size` to specify a custom chunk size.
    pub fn new() -> Self {
        PhysicsTilemap {
            storage: ChunkedStorage::default(),
            spawn_queue: Vec::new(),
            data: ChunkedStorage::default(),
        }
    }

    /// Create a new physics tilemap with a custom chunk size.
    pub fn new_with_chunk_size(chunk_size: u32) -> Self {
        PhysicsTilemap {
            storage: ChunkedStorage::new(chunk_size),
            spawn_queue: Vec::new(),
            data: ChunkedStorage::new(chunk_size),
        }
    }

    /// Get a tile.
    #[inline]
    pub fn get(&self, index: IVec2) -> Option<Entity> {
        self.storage.get_elem(index).cloned()
    }

    /// Set a tile. This actually queues the tile and it will be spawned later.
    #[inline]
    pub fn set(&mut self, index: IVec2, tile: PhysicsTile) {
        self.spawn_queue
            .push((IRect::from_center_size(index, IVec2::ZERO), tile, None));
    }

    /// Remove a tile.
    #[inline]
    pub fn remove(&mut self, commands: &mut Commands, index: IVec2) {
        if let Some(entity) = self.storage.remove_elem(index) {
            commands.entity(entity).despawn();
        }
    }

    /// Remove a chunk.
    #[inline]
    pub fn remove_chunk(&mut self, commands: &mut Commands, index: IVec2) {
        if let Some(chunk) = self.storage.remove_chunk(index) {
            chunk.into_iter().filter_map(|e| e).for_each(|entity| {
                commands.entity(entity).despawn();
            });
        }
    }

    /// Remove all tiles.
    #[inline]
    pub fn remove_all(&mut self, commands: &mut Commands) {
        for entity in self.storage.iter_some() {
            commands.entity(*entity).despawn();
        }
        self.storage.clear();
    }

    /// Fill a rectangle area with the same tile.
    ///
    /// Set `concat` to true if you want to concat the adjacent tiles.
    pub fn fill_rect(&mut self, area: TileArea, tile: PhysicsTile, concat: bool) {
        if concat {
            self.spawn_queue
                .push((IRect::from_corners(area.origin, area.dest), tile, None));
        } else {
            self.spawn_queue.extend(
                (area.origin.y..=area.dest.y)
                    .flat_map(|y| (area.origin.x..=area.dest.x).map(move |x| IVec2 { x, y }))
                    .map(|index| {
                        (
                            IRect::from_center_size(index, IVec2::ZERO),
                            tile.clone(),
                            None,
                        )
                    }),
            );
        }
    }

    /// Fill a rectangle area with tiles returned by `physics_tile`.
    /// This won't concat the adjacent tiles.
    ///
    /// Set `relative_index` to true if your function takes index relative to the area origin.
    pub fn fill_rect_custom(
        &mut self,
        area: TileArea,
        physics_tile: impl Fn(IVec2) -> Option<PhysicsTile>,
        relative_index: bool,
    ) {
        self.spawn_queue.reserve(area.size());
        for y in area.origin.y..=area.dest.y {
            for x in area.origin.x..=area.dest.x {
                let index = IVec2 { x, y };
                if let Some(tile) = physics_tile(if relative_index {
                    index - area.origin
                } else {
                    index
                }) {
                    self.spawn_queue.push((
                        IRect::from_center_size(index, IVec2::ZERO),
                        tile,
                        None,
                    ));
                }
            }
        }
    }

    /// Fill a rectangle area with tiles from a buffer. This can be faster than setting them one by one.
    pub fn fill_with_buffer(&mut self, origin: IVec2, buffer: PhysicsTileBuffer) {
        self.spawn_queue
            .extend(buffer.tiles.into_iter().map(|(index, tile)| {
                (
                    IRect::from_center_size(index + origin, IVec2::ZERO),
                    tile,
                    None,
                )
            }));
    }

    pub fn fill_with_buffer_packed(&mut self, origin: IVec2, buffer: PackedPhysicsTileBuffer) {
        self.spawn_queue
            .extend(buffer.tiles.into_iter().map(|(index, tile)| {
                (
                    IRect::from_center_size(index + origin, IVec2::ZERO),
                    tile.into(),
                    None,
                )
            }));
    }
}
