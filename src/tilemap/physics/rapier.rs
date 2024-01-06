use bevy::{
    app::{FixedUpdate, Plugin},
    ecs::{
        event::{EventReader, EventWriter},
        system::{Commands, Query},
    },
    math::IVec2,
};
use bevy_rapier2d::{
    geometry::{ActiveEvents, Collider, Friction, Sensor},
    pipeline::CollisionEvent,
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

pub struct PhysicsRapierTilemapPlugin;

impl Plugin for PhysicsRapierTilemapPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(FixedUpdate, collision_handler);
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
    tile_slot_size: &TilemapSlotSize,
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
                tile_slot_size,
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
    tile_slot_size: &TilemapSlotSize,
) {
    let Some(tile_entity) = storage.get(index) else {
        return;
    };
    let mut tile_entity = commands.entity(tile_entity);

    // Seems like rapier will only take the entity's transform into consideration.
    // So we need to use the world position.
    let collider = Collider::convex_hull(&coordinates::get_tile_convex_hull_world(
        index,
        ty,
        transform,
        tile_pivot,
        tile_slot_size,
    ))
    .unwrap();

    if is_trigger {
        tile_entity.insert((collider, Sensor, ActiveEvents::COLLISION_EVENTS));
    } else {
        tile_entity.insert((collider, ActiveEvents::COLLISION_EVENTS));
    }

    if let Some(coe) = friction {
        tile_entity.insert(Friction::coefficient(coe));
    }
}

pub fn collision_handler(
    mut collision: EventReader<CollisionEvent>,
    mut tile_collision: EventWriter<TileCollision>,
    tiles_query: Query<&Tile>,
) {
    let mut colls = Vec::with_capacity(collision.len());
    for c in collision.read() {
        match c {
            CollisionEvent::Started(e1, e2, _) => {
                if let Some(data) = get_collision(*e1, *e2, &tiles_query) {
                    colls.push(TileCollision::Started(data));
                }
            }
            CollisionEvent::Stopped(e1, e2, _) => {
                if let Some(data) = get_collision(*e1, *e2, &tiles_query) {
                    colls.push(TileCollision::Stopped(data));
                }
            }
        }
    }
    tile_collision.send_batch(colls);
}
