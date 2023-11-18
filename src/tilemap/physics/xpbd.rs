use bevy::{
    app::{FixedUpdate, Plugin},
    ecs::{
        event::{EventReader, EventWriter},
        system::{Commands, Query},
    },
    math::{UVec2, Vec2},
};
use bevy_xpbd_2d::{
    components::{Collider, Friction, RigidBody},
    plugins::{collision::contact_reporting::Collision, PhysicsDebugPlugin, PhysicsPlugins},
    resources::Gravity,
};

use crate::{
    math::FillArea,
    tilemap::{Tile, TileType, Tilemap},
};

use super::TileCollision;

pub struct PhysicsXpbdTilemapPlugin;

impl Plugin for PhysicsXpbdTilemapPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(PhysicsPlugins::default())
            .add_systems(FixedUpdate, collision_handler)
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
        has_rigid_body: bool,
    ) {
        for y in area.origin.y..=area.dest.y {
            for x in area.origin.x..=area.dest.x {
                self.set_physics_tile_xpbd(commands, UVec2 { x, y }, friction, has_rigid_body);
            }
        }
    }

    /// Give the tile a physics collider.
    pub fn set_physics_tile_xpbd(
        &mut self,
        commands: &mut Commands,
        index: UVec2,
        friction: Option<f32>,
        has_rigid_body: bool,
    ) {
        let Some(tile_entity) = self.get(index) else {
            return;
        };

        let x = self.tile_render_size.x;
        let y = self.tile_render_size.y;
        let translation = self.index_to_world(index);

        let collider = match self.tile_type {
            TileType::Square => Collider::convex_decomposition(
                vec![
                    Vec2::new(-x / 2., -y / 2.),
                    Vec2::new(-x / 2., y / 2.),
                    Vec2::new(x / 2., y / 2.),
                    Vec2::new(x / 2., -y / 2.),
                ]
                .into_iter()
                .map(|p| p + translation)
                .collect(),
                vec![[0, 1], [1, 2], [2, 3], [3, 0]],
            ),
            TileType::IsometricDiamond => Collider::convex_decomposition(
                vec![
                    Vec2::new(-x / 2., 0.),
                    Vec2::new(0., y / 2.),
                    Vec2::new(x / 2., 0.),
                    Vec2::new(0., -y / 2.),
                ]
                .into_iter()
                .map(|p| p + translation)
                .collect(),
                vec![[0, 1], [1, 2], [2, 3], [3, 0]],
            ),
        };

        if has_rigid_body {
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
    mut collision: EventReader<Collision>,
    mut tile_collision: EventWriter<TileCollision>,
    tiles_query: Query<&Tile>,
) {
    let mut colls = Vec::with_capacity(collision.len());
    for c in collision.read() {
        let tile;
        let tile_entity;
        let collider_entity;
        if let Ok(t) = tiles_query.get(c.0.entity1) {
            tile = t;
            tile_entity = c.0.entity1;
            collider_entity = c.0.entity2;
        } else if let Ok(t) = tiles_query.get(c.0.entity2) {
            tile = t;
            tile_entity = c.0.entity2;
            collider_entity = c.0.entity1;
        } else {
            continue;
        }
        colls.push(TileCollision {
            tile_index: tile.index,
            tile_entity,
            tile_snapshot: tile.clone(),
            collider_entity,
        });
    }
    tile_collision.send_batch(colls);
}
