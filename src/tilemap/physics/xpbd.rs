use bevy::{
    app::{Plugin, Update},
    ecs::{
        event::{EventReader, EventWriter},
        system::{Commands, Query},
    },
    math::{UVec2, Vec2},
};
use bevy_xpbd_2d::{
    components::{Collider, Friction, RigidBody},
    plugins::{
        collision::contact_reporting::{CollisionEnded, CollisionStarted},
        PhysicsDebugPlugin, PhysicsPlugins,
    },
    resources::Gravity,
};

use crate::{
    math::FillArea,
    tilemap::{
        map::Tilemap,
        tile::{Tile, TileType},
    },
};

use super::{get_collision, TileCollision};

pub struct PhysicsXpbdTilemapPlugin;

impl Plugin for PhysicsXpbdTilemapPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(PhysicsPlugins::default())
            .add_systems(Update, collision_handler)
            .insert_resource(Gravity(Vec2::ZERO));

        #[cfg(feature = "debug")]
        app.add_plugins(PhysicsDebugPlugin::default());
    }
}

impl Tilemap {
    /// Give the tiles physics colliders.
    pub fn fill_physics_tile_xpbd(
        &mut self,
        commands: &mut Commands,
        area: FillArea,
        friction: Option<f32>,
        is_trigger: bool,
    ) {
        for y in area.origin.y..=area.dest.y {
            for x in area.origin.x..=area.dest.x {
                self.set_physics_tile_xpbd(commands, UVec2 { x, y }, friction, is_trigger);
            }
        }
    }

    /// Give the tile a physics collider.
    pub fn set_physics_tile_xpbd(
        &mut self,
        commands: &mut Commands,
        index: UVec2,
        friction: Option<f32>,
        is_trigger: bool,
    ) {
        let Some(tile_entity) = self.get(index) else {
            return;
        };

        let x = self.tile_render_scale.x;
        let y = self.tile_render_scale.y;
        let translation = self.index_to_world(index);

        let collider = match self.tile_type {
            TileType::Square => Collider::convex_hull(
                vec![
                    Vec2::new(-x / 2., -y / 2.),
                    Vec2::new(-x / 2., y / 2.),
                    Vec2::new(x / 2., y / 2.),
                    Vec2::new(x / 2., -y / 2.),
                ]
                .into_iter()
                .map(|p| p + translation)
                .collect(),
            ),
            TileType::IsometricDiamond => Collider::convex_hull(
                vec![
                    Vec2::new(-x / 2., 0.),
                    Vec2::new(0., y / 2.),
                    Vec2::new(x / 2., 0.),
                    Vec2::new(0., -y / 2.),
                ]
                .into_iter()
                .map(|p| p + translation)
                .collect(),
            ),
        }
        .unwrap();

        if is_trigger {
            commands.entity(tile_entity).insert(collider);
        } else {
            commands
                .entity(tile_entity)
                .insert((collider, RigidBody::Static));
        }

        if let Some(coe) = friction {
            commands.entity(tile_entity).insert(Friction::new(coe));
        }
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
