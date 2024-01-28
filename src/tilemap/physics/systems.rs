use bevy::{
    ecs::{
        entity::Entity,
        system::{ParallelCommands, Query},
    },
    math::UVec2,
};

use crate::{
    math::aabb::IAabb2d,
    tilemap::{
        chunking::storage::ChunkedStorage,
        coordinates,
        map::{TilePivot, TilemapSlotSize, TilemapTransform, TilemapType},
    },
};

use super::{DataPhysicsTilemap, PackedPhysicsTile, PhysicsTilemap};

pub fn spawn_colliders(
    commands: ParallelCommands,
    mut tilemaps_query: Query<(
        &mut PhysicsTilemap,
        &TilemapType,
        &TilemapTransform,
        &TilePivot,
        &TilemapSlotSize,
    )>,
) {
    tilemaps_query.par_iter_mut().for_each(
        |(mut physics_tilemap, ty, transform, tile_pivot, slot_size)| {
            let physics_tiles = physics_tilemap.spawn_queue.drain(..).collect::<Vec<_>>();
            physics_tiles.into_iter().for_each(|(aabb, physics_tile)| {
                commands.command_scope(|mut c| {
                    let vertices = coordinates::get_tile_collider_world(
                        aabb.min,
                        *ty,
                        aabb.size().as_uvec2(),
                        transform,
                        tile_pivot.0,
                        slot_size.0,
                    );

                    let packed_tile = PackedPhysicsTile {
                        parent: aabb.min,
                        collider: vertices.clone(),
                        physics_tile,
                    };

                    physics_tilemap
                        .storage
                        .set_elem(aabb.min, packed_tile.spawn(&mut c, *ty));
                    physics_tilemap.data.set_elem(aabb.min, packed_tile);
                });
            });
        },
    );
}

pub fn data_physics_tilemap_analyzer(
    commands: ParallelCommands,
    mut tilemaps_query: Query<(Entity, &mut DataPhysicsTilemap, Option<&mut PhysicsTilemap>)>,
) {
    tilemaps_query
        .par_iter_mut()
        .for_each(|(entity, mut data_tilemap, mut physics_tilemap)| {
            let mut aabbs = Vec::new();
            let size = data_tilemap.size;
            let air = data_tilemap.air;

            for y in 0..size.y {
                for x in 0..size.x {
                    let cur = UVec2 { x, y };

                    let cur_i = {
                        let i = data_tilemap.get_or_air(cur);
                        if i == air {
                            continue;
                        }
                        i
                    };

                    let mut d = UVec2 {
                        x: if x == size.x - 1 { 0 } else { 1 },
                        y: if y == size.y - 1 { 0 } else { 1 },
                    };
                    let mut dst = cur;
                    while d.x != 0 || d.y != 0 {
                        for t_x in cur.x..=dst.x {
                            if data_tilemap.get_or_air(UVec2::new(t_x, dst.y + d.y)) != cur_i {
                                d.y = 0;
                                break;
                            }
                        }

                        for t_y in cur.y..=dst.y {
                            if data_tilemap.get_or_air(UVec2::new(dst.x + d.x, t_y)) != cur_i {
                                d.x = 0;
                                break;
                            }
                        }

                        if d == UVec2::ONE
                            && data_tilemap.get_or_air(UVec2::new(dst.x + 1, dst.y + 1)) != cur_i
                        {
                            d.y = 0;
                        }

                        dst += d;
                    }

                    for y in cur.y..=dst.y {
                        for x in cur.x..=dst.x {
                            data_tilemap.set(UVec2 { x, y }, air);
                        }
                    }

                    aabbs.push((
                        IAabb2d {
                            min: cur.as_ivec2() + data_tilemap.origin,
                            max: dst.as_ivec2() + data_tilemap.origin,
                        },
                        data_tilemap.get_tile(cur_i).unwrap_or_default(),
                    ));
                }
            }

            commands.command_scope(|mut c| {
                if let Some(physics_tilemap) = &mut physics_tilemap {
                    physics_tilemap.spawn_queue.extend(aabbs);
                } else {
                    c.entity(entity).insert(PhysicsTilemap {
                        storage: Default::default(),
                        spawn_queue: aabbs,
                        data: ChunkedStorage::default(),
                    });
                }

                c.entity(entity).remove::<DataPhysicsTilemap>();
            });
        });
}
