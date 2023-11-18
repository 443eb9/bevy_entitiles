use bevy::{
    app::{FixedUpdate, Plugin},
    ecs::{
        event::{EventReader, EventWriter},
        system::{Commands, Query},
    },
    math::{UVec2, Vec2},
};
use bevy_rapier2d::{
    dynamics::RigidBody,
    geometry::{Collider, Friction},
    pipeline::ContactForceEvent,
    plugin::{NoUserData, RapierPhysicsPlugin},
    render::RapierDebugRenderPlugin,
};

use crate::{
    math::FillArea,
    tilemap::{Tile, TileType, Tilemap},
};

use super::TileCollision;

pub struct PhysicsRapierTilemapPlugin;

impl Plugin for PhysicsRapierTilemapPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
            .add_systems(FixedUpdate, collision_handler);

        #[cfg(feature = "debug")]
        app.add_plugins(RapierDebugRenderPlugin::default());
    }
}

impl Tilemap {
    /// Give the tiles physics colliders.
    pub fn fill_physics_tile_rapier(
        &mut self,
        commands: &mut Commands,
        area: FillArea,
        friction: Option<f32>,
        has_rigid_body: bool,
    ) {
        for y in area.origin.y..=area.dest.y {
            for x in area.origin.x..=area.dest.x {
                self.set_physics_tile_rapier(commands, UVec2 { x, y }, friction, has_rigid_body);
            }
        }
    }

    /// Give the tile a physics collider.
    pub fn set_physics_tile_rapier(
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
                &[
                    (Vec2::new(-x / 2., -y / 2.) + translation).into(),
                    (Vec2::new(-x / 2., y / 2.) + translation).into(),
                    (Vec2::new(x / 2., y / 2.) + translation).into(),
                    (Vec2::new(x / 2., -y / 2.) + translation).into(),
                ],
                &[[0, 1], [1, 2], [2, 3], [3, 0]],
            ),
            TileType::IsometricDiamond => Collider::convex_decomposition(
                &[
                    (Vec2::new(-x / 2., 0.) + translation).into(),
                    (Vec2::new(0., y / 2.) + translation).into(),
                    (Vec2::new(x / 2., 0.) + translation).into(),
                    (Vec2::new(0., -y / 2.) + translation).into(),
                ],
                &[[0, 1], [1, 2], [2, 3], [3, 0]],
            ),
        };

        if has_rigid_body {
            commands.entity(tile_entity).insert(collider);
        } else {
            commands
                .entity(tile_entity)
                .insert(collider)
                .insert(RigidBody::Fixed);
        }

        if let Some(coe) = friction {
            commands
                .entity(tile_entity)
                .insert(Friction::coefficient(coe));
        }
    }
}

pub fn collision_handler(
    mut collision: EventReader<ContactForceEvent>,
    mut tile_collision: EventWriter<TileCollision>,
    tiles_query: Query<&Tile>,
) {
    let mut colls = Vec::with_capacity(collision.len());
    for c in collision.read() {
        let tile;
        let tile_entity;
        let collider_entity;
        if let Ok(t) = tiles_query.get(c.collider1) {
            tile = t;
            tile_entity = c.collider1;
            collider_entity = c.collider2;
        } else if let Ok(t) = tiles_query.get(c.collider2) {
            tile = t;
            tile_entity = c.collider2;
            collider_entity = c.collider1;
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
