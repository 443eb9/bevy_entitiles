use bevy::ecs::system::{ParallelCommands, Query};
use bevy_xpbd_2d::components::{Collider, Friction, RigidBody};

use crate::tilemap::{
    coordinates,
    map::{TilePivot, TilemapSlotSize, TilemapStorage, TilemapTransform, TilemapType},
};

use super::PhysicsTilemap;

pub fn spawn_colliders(
    commands: ParallelCommands,
    mut tilemaps_query: Query<(
        &mut PhysicsTilemap,
        &TilemapStorage,
        &TilemapType,
        &TilemapTransform,
        &TilePivot,
        &TilemapSlotSize,
    )>,
) {
    tilemaps_query.par_iter_mut().for_each(
        |(mut physics_tilemap, storage, ty, transform, tile_pivot, slot_size)| {
            let physics_tiles = physics_tilemap.spawn_queue.drain(..).collect::<Vec<_>>();
            physics_tiles.into_iter().for_each(|(index, physics_tile)| {
                let Some(tile_entity) = storage.get(index) else {
                    return;
                };

                physics_tilemap.storage.set_elem(index, Some(tile_entity));

                commands.command_scope(|mut c| {
                    let mut tile_entity = c.entity(tile_entity);

                    let collider = Collider::convex_hull(coordinates::get_tile_convex_hull_world(
                        index, ty, transform, tile_pivot, slot_size,
                    ))
                    .unwrap();

                    if physics_tile.rigid_body {
                        tile_entity.insert((collider, RigidBody::Static));
                    } else {
                        tile_entity.insert(collider);
                    }

                    if let Some(coe) = physics_tile.friction {
                        tile_entity.insert(Friction::new(coe));
                    }
                });
            });
        },
    );
}
