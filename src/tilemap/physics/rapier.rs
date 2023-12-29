use bevy::{
    app::{FixedUpdate, Plugin},
    ecs::{
        event::{EventReader, EventWriter},
        system::{Commands, Query},
    },
    math::UVec2,
};
use bevy_rapier2d::{
    geometry::{ActiveEvents, Collider, Friction, Sensor},
    pipeline::CollisionEvent,
};

use crate::{
    math::FillArea,
    tilemap::{map::Tilemap, tile::Tile},
};

use super::{get_collision, TileCollision};

pub struct PhysicsRapierTilemapPlugin;

impl Plugin for PhysicsRapierTilemapPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(FixedUpdate, collision_handler);
    }
}

impl Tilemap {
    /// Give the tiles physics colliders.
    pub fn fill_physics_tile_rapier(
        &mut self,
        commands: &mut Commands,
        area: FillArea,
        friction: Option<f32>,
        is_trigger: bool,
    ) {
        for y in area.origin.y..=area.dest.y {
            for x in area.origin.x..=area.dest.x {
                self.set_physics_tile_rapier(commands, UVec2 { x, y }, friction, is_trigger);
            }
        }
    }

    /// Give the tile a physics collider.
    pub fn set_physics_tile_rapier(
        &mut self,
        commands: &mut Commands,
        index: UVec2,
        friction: Option<f32>,
        is_trigger: bool,
    ) {
        let Some(tile_entity) = self.get(index) else {
            return;
        };

        let collider = Collider::convex_hull(&self.get_tile_convex_hull(index)).unwrap();

        if is_trigger {
            commands
                .entity(tile_entity)
                .insert((collider, Sensor, ActiveEvents::COLLISION_EVENTS));
        } else {
            commands
                .entity(tile_entity)
                .insert((collider, ActiveEvents::COLLISION_EVENTS));
        }

        if let Some(coe) = friction {
            commands
                .entity(tile_entity)
                .insert(Friction::coefficient(coe));
        }
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
