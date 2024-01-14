use bevy::{
    app::{App, Plugin, Update},
    ecs::{
        component::Component,
        entity::Entity,
        event::{Event, EventReader, EventWriter},
        system::{Commands, Query},
    },
    math::{IVec2, Vec2},
    reflect::Reflect,
};
use bevy_xpbd_2d::{
    components::{Collider, Friction, RigidBody},
    plugins::collision::contact_reporting::{CollisionEnded, CollisionStarted},
};

use crate::{math::TileArea, tilemap::tile::Tile, DEFAULT_CHUNK_SIZE};

use super::{
    buffers::{PhysicsTilesBuffer, Tiles},
    chunking::storage::ChunkedStorage,
    map::TilemapStorage,
};

pub mod systems;

pub struct EntiTilesPhysicsTilemapPlugin;

impl Plugin for EntiTilesPhysicsTilemapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (collision_handler, systems::spawn_colliders));

        app.add_event::<TileCollision>();

        app.register_type::<TileCollision>()
            .register_type::<CollisionData>()
            .register_type::<PhysicsTilemap>()
            .register_type::<PhysicsTile>();
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub struct PackedPhysicsTile {
    pub parent: IVec2,
    pub collider: Vec<Vec2>,
    pub physics_tile: PhysicsTile,
}

impl PackedPhysicsTile {
    pub fn spawn(&self, commands: &mut Commands, storage: &TilemapStorage) -> Option<Entity> {
        storage.get(self.parent).map(|e| {
            let mut entity = commands.entity(e);
            entity.insert(Collider::convex_hull(self.collider.clone()).unwrap());
            if self.physics_tile.rigid_body {
                entity.insert(RigidBody::Static);
            }
            if let Some(friction) = &self.physics_tile.friction {
                entity.insert(Friction::new(*friction));
            }
            e
        })
    }
}

#[derive(Debug, Clone, Reflect)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub struct PhysicsTile {
    pub rigid_body: bool,
    pub friction: Option<f32>,
}

impl Tiles for PhysicsTile {}

#[derive(Component, Debug, Clone, Reflect)]
pub struct PhysicsTilemap {
    pub(crate) storage: ChunkedStorage<Entity>,
    pub(crate) spawn_queue: Vec<(IVec2, PhysicsTile)>,
    pub(crate) concat_queue: Vec<(IVec2, PhysicsTile)>,
    pub(crate) split_queue: Vec<IVec2>,
}

impl PhysicsTilemap {
    pub fn new() -> Self {
        PhysicsTilemap {
            storage: ChunkedStorage::new(DEFAULT_CHUNK_SIZE),
            spawn_queue: Vec::new(),
            concat_queue: Vec::new(),
            split_queue: Vec::new(),
        }
    }

    pub fn new_with_chunk_size(chunk_size: u32) -> Self {
        PhysicsTilemap {
            storage: ChunkedStorage::new(chunk_size),
            spawn_queue: Vec::new(),
            concat_queue: Vec::new(),
            split_queue: Vec::new(),
        }
    }

    /// Get a tile.
    ///
    /// **Notice** This tile entity is the parent of the collider.
    /// And it's shared with the `Tile` entity.
    #[inline]
    pub fn get(&self, index: IVec2) -> Option<Entity> {
        self.storage.get_elem(index).cloned()
    }

    /// Set a tile. This actually queues the tile and it will be spawned later.
    #[inline]
    pub fn set(&mut self, index: IVec2, tile: PhysicsTile) {
        self.spawn_queue.push((index, tile));
    }

    /// Fill a rectangle area with the same tile.
    /// This won't concat the adjacent tiles.
    pub fn fill_rect(&mut self, area: TileArea, tile: PhysicsTile) {
        self.spawn_queue.extend(
            (area.origin.y..=area.dest.y)
                .flat_map(|y| (area.origin.x..=area.dest.x).map(move |x| IVec2 { x, y }))
                .map(|index| (index, tile.clone())),
        );
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
                if let Some(tile) = physics_tile(if relative_index {
                    IVec2 { x, y } - area.origin
                } else {
                    IVec2 { x, y }
                }) {
                    self.spawn_queue.push((IVec2 { x, y }, tile));
                }
            }
        }
    }

    /// Fill a rectangle area with tiles from a buffer. This can be faster than setting them one by one.
    pub fn fill_with_buffer(&mut self, origin: IVec2, buffer: PhysicsTilesBuffer) {
        self.spawn_queue.extend(
            buffer
                .tiles
                .into_iter()
                .map(|(index, tile)| (index + origin, tile)),
        );
    }

    /// Try to concat all the tiles that are adjacent to the given origin.
    /// And replace them with the lowest number of colliders.
    #[inline]
    pub fn concat(&mut self, origin: IVec2, tile: PhysicsTile) {
        self.concat_queue.push((origin, tile));
    }

    /// Try to split the concat tile at the given parent index.
    ///
    /// The index should be the `origin` you entered in `concat()`.
    #[inline]
    pub fn split(&mut self, index: IVec2) {
        self.split_queue.push(index);
    }
}

#[derive(Event, Debug, Reflect)]
pub enum TileCollision {
    Started(CollisionData),
    Stopped(CollisionData),
}

#[derive(Debug, Reflect, Clone)]
pub struct CollisionData {
    pub tile_index: IVec2,
    pub tile_entity: Entity,
    pub collider_entity: Entity,
}

fn get_collision(e1: Entity, e2: Entity, query: &Query<&Tile>) -> Option<CollisionData> {
    let (e_tile, e_other, tile) = {
        if let Ok(t) = query.get(e1) {
            (e1, e2, t)
        } else if let Ok(t) = query.get(e2) {
            (e2, e1, t)
        } else {
            return None;
        }
    };

    Some(CollisionData {
        tile_index: tile.index,
        tile_entity: e_tile,
        collider_entity: e_other,
    })
}

pub fn collision_handler(
    mut collision_start: EventReader<CollisionStarted>,
    mut collision_end: EventReader<CollisionEnded>,
    mut tile_collision: EventWriter<TileCollision>,
    tiles_query: Query<&Tile>,
) {
    let mut colls = Vec::with_capacity(collision_start.len());
    for c in collision_start.read() {
        if let Some(data) = get_collision(c.0, c.1, &tiles_query) {
            colls.push(TileCollision::Started(data));
        }
    }
    for c in collision_end.read() {
        if let Some(data) = get_collision(c.0, c.1, &tiles_query) {
            colls.push(TileCollision::Stopped(data));
        }
    }
    tile_collision.send_batch(colls);
}
