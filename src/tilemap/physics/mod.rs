use bevy::{
    app::{App, Plugin, Update},
    ecs::{
        component::Component,
        entity::Entity,
        event::{Event, EventReader, EventWriter},
        system::{Commands, Query},
    },
    math::{IVec2, UVec2, Vec2},
    reflect::Reflect,
    utils::HashMap,
};
use bevy_xpbd_2d::{
    components::{Collider, Friction, RigidBody},
    plugins::collision::contact_reporting::{CollisionEnded, CollisionStarted},
};

use crate::{
    math::{aabb::IAabb2d, TileArea},
    tilemap::tile::Tile,
};

use super::{
    buffers::{PhysicsTileBuffer, Tiles},
    chunking::storage::{ChunkedStorage, EntityChunkedStorage, PackedPhysicsTileChunkedStorage},
    map::TilemapType,
};

pub mod systems;

pub struct EntiTilesPhysicsTilemapPlugin;

impl Plugin for EntiTilesPhysicsTilemapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                collision_handler,
                systems::spawn_colliders,
                systems::data_physics_tilemap_analyzer,
            ),
        );

        app.add_event::<TileCollision>();

        app.register_type::<TileCollision>()
            .register_type::<CollisionData>()
            .register_type::<PhysicsTilemap>()
            .register_type::<DataPhysicsTilemap>()
            .register_type::<PhysicsTile>();
    }
}

#[derive(Debug, Clone, Reflect)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub struct PackedPhysicsTile {
    pub parent: IVec2,
    pub collider: Vec<Vec2>,
    pub physics_tile: PhysicsTile,
}

impl Tiles for PackedPhysicsTile {}

impl PackedPhysicsTile {
    pub fn spawn(&self, commands: &mut Commands, ty: TilemapType) -> Entity {
        let mut entity = commands.spawn(match ty {
            TilemapType::Square | TilemapType::Isometric => {
                Collider::convex_hull(self.collider.clone()).unwrap()
            }
            TilemapType::Hexagonal(_) => Collider::polyline(self.collider.clone(), None),
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

#[derive(Component, Debug, Clone, Reflect)]
pub struct DataPhysicsTilemap {
    pub(crate) origin: IVec2,
    pub(crate) data: Vec<i32>,
    pub(crate) size: UVec2,
    pub(crate) air: i32,
    pub(crate) tiles: HashMap<i32, PhysicsTile>,
}

impl DataPhysicsTilemap {
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

    #[inline]
    pub fn get_or_air(&self, index: UVec2) -> i32 {
        self.data
            .get((index.x + index.y * self.size.x) as usize)
            .cloned()
            .unwrap_or(self.air)
    }

    #[inline]
    pub fn get_tile(&self, value: i32) -> Option<PhysicsTile> {
        self.tiles.get(&value).cloned()
    }

    #[inline]
    pub fn set(&mut self, index: UVec2, value: i32) {
        self.data[(index.x + index.y * self.size.x) as usize] = value;
    }
}

#[derive(Component, Debug, Clone, Reflect)]
pub struct PhysicsTilemap {
    pub(crate) storage: EntityChunkedStorage,
    pub(crate) spawn_queue: Vec<(IAabb2d, PhysicsTile)>,
    pub(crate) data: PackedPhysicsTileChunkedStorage,
}

impl PhysicsTilemap {
    pub fn new() -> Self {
        PhysicsTilemap {
            storage: ChunkedStorage::default(),
            spawn_queue: Vec::new(),
            data: ChunkedStorage::default(),
        }
    }

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
        self.spawn_queue.push((IAabb2d::splat(index), tile));
    }

    #[inline]
    pub fn remove(&mut self, commands: &mut Commands, index: IVec2) {
        if let Some(entity) = self.storage.remove_elem(index) {
            commands.entity(entity).despawn();
        }
    }

    #[inline]
    pub fn remove_chunk(&mut self, commands: &mut Commands, index: IVec2) {
        if let Some(chunk) = self.storage.remove_chunk(index) {
            chunk.into_iter().filter_map(|e| e).for_each(|entity| {
                commands.entity(entity).despawn();
            });
        }
    }

    #[inline]
    pub fn remove_all(&mut self, commands: &mut Commands) {
        for entity in self.storage.iter_some() {
            commands.entity(*entity).despawn();
        }
        self.storage.clear();
    }

    /// Fill a rectangle area with the same tile.
    /// This won't concat the adjacent tiles.
    pub fn fill_rect(&mut self, area: TileArea, tile: PhysicsTile, concat: bool) {
        if concat {
            self.spawn_queue.push((area.into(), tile));
        } else {
            self.spawn_queue.extend(
                (area.origin.y..=area.dest.y)
                    .flat_map(|y| (area.origin.x..=area.dest.x).map(move |x| IVec2 { x, y }))
                    .map(|index| (IAabb2d::splat(index), tile.clone())),
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
                    self.spawn_queue.push((IAabb2d::splat(index), tile));
                }
            }
        }
    }

    /// Fill a rectangle area with tiles from a buffer. This can be faster than setting them one by one.
    pub fn fill_with_buffer(&mut self, origin: IVec2, buffer: PhysicsTileBuffer) {
        self.spawn_queue.extend(
            buffer
                .tiles
                .into_iter()
                .map(|(index, tile)| (IAabb2d::splat(index + origin), tile)),
        );
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
