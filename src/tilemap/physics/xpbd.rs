use bevy::{
    app::{Plugin, Update},
    ecs::{
        event::{EventReader, EventWriter},
        system::{Commands, Query},
    },
    math::IVec2,
};
use bevy_xpbd_2d::{
    components::{Collider, Friction, RigidBody},
    plugins::collision::contact_reporting::{CollisionEnded, CollisionStarted},
};

use crate::{
    math::TileArea,
    tilemap::{
        coordinates,
        map::{TilePivot, TilemapSlotSize, TilemapStorage, TilemapTransform, TilemapType},
        tile::Tile,
    },
};

use super::{get_collision, TileCollision};

pub struct PhysicsXpbdTilemapPlugin;

impl Plugin for PhysicsXpbdTilemapPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Update, collision_handler);
    }
}

/// Give the tiles physics colliders.
pub fn fill_physics_tile(
    commands: &mut Commands,
    area: TileArea,
    friction: Option<f32>,
    is_trigger: bool,
    storage: &TilemapStorage,
    ty: &TilemapType,
    transform: &TilemapTransform,
    tile_pivot: &TilePivot,
    slot_size: &TilemapSlotSize,
) {
    for y in area.origin.y..=area.dest.y {
        for x in area.origin.x..=area.dest.x {
            set_physics_tile(
                commands,
                IVec2 { x, y },
                friction,
                is_trigger,
                storage,
                ty,
                transform,
                tile_pivot,
                slot_size,
            );
        }
    }
}

/// Give the tile a physics collider.
pub fn set_physics_tile(
    commands: &mut Commands,
    index: IVec2,
    friction: Option<f32>,
    is_trigger: bool,
    storage: &TilemapStorage,
    ty: &TilemapType,
    transform: &TilemapTransform,
    tile_pivot: &TilePivot,
    slot_size: &TilemapSlotSize,
) {
    let Some(tile_entity) = storage.get(index) else {
        return;
    };
    let mut tile_entity = commands.entity(tile_entity);

    // Seems like rapier will only take the entity's transform into consideration.
    // So we need to use the world position.
    let collider = Collider::convex_hull(coordinates::get_tile_convex_hull_world(
        index,
        ty,
        transform,
        tile_pivot,
        slot_size,
    ))
    .unwrap();

    if is_trigger {
        tile_entity.insert(collider);
    } else {
        tile_entity.insert((collider, RigidBody::Static));
    }

    if let Some(coe) = friction {
        tile_entity.insert(Friction::new(coe));
    }
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
