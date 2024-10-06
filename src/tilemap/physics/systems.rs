use bevy::{
    ecs::{entity::Entity, event::EventWriter, system::Query},
    math::UVec2,
    prelude::Commands,
};

use crate::{
    math::TileArea,
    tilemap::{
        chunking::storage::ChunkedStorage,
        coordinates,
        map::{TilePivot, TilemapSlotSize, TilemapTransform, TilemapType},
        physics::{
            DataPhysicsTilemap, PackedPhysicsTile, PhysicsCollider, PhysicsTileSpawn,
            PhysicsTilemap,
        },
    },
};

pub fn spawn_colliders(
    mut commands: Commands,
    mut tilemaps_query: Query<(
        Entity,
        &mut PhysicsTilemap,
        &TilemapType,
        &TilemapTransform,
        &TilePivot,
        &TilemapSlotSize,
    )>,
    mut spawn_event: EventWriter<PhysicsTileSpawn>,
) {
    for (entity, mut physics_tilemap, ty, transform, tile_pivot, slot_size) in &mut tilemaps_query {
        let PhysicsTilemap {
            storage,
            spawn_queue,
            data,
        } = &mut *physics_tilemap;

        for (aabb, physics_tile, maybe_int_repr) in spawn_queue.drain(..) {
            let vertices = coordinates::get_tile_collider_world(
                aabb.origin,
                *ty,
                aabb.extent,
                transform,
                tile_pivot.0,
                slot_size.0,
            );

            let packed_tile = PackedPhysicsTile {
                parent: aabb.origin,
                collider: match ty {
                    TilemapType::Square | TilemapType::Isometric => {
                        PhysicsCollider::Convex(vertices)
                    }
                    TilemapType::Hexagonal(_) => PhysicsCollider::Polyline(vertices),
                },
                physics_tile,
            };
            let tile_entity = packed_tile.spawn(&mut commands);

            spawn_event.send(PhysicsTileSpawn {
                tilemap: entity,
                tile: tile_entity,
                int_repr: maybe_int_repr,
            });

            storage.set_elem(aabb.origin, tile_entity);
            data.set_elem(aabb.origin, packed_tile);
        }
    }
}

pub fn data_physics_tilemap_analyzer(
    mut commands: Commands,
    mut tilemaps_query: Query<(Entity, &mut DataPhysicsTilemap, Option<&mut PhysicsTilemap>)>,
) {
    for (entity, mut data_tilemap, mut physics_tilemap) in &mut tilemaps_query {
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
                    TileArea::from_min_max(
                        cur.as_ivec2() + data_tilemap.origin,
                        dst.as_ivec2() + data_tilemap.origin,
                    ),
                    data_tilemap.get_tile(cur_i).unwrap_or_default(),
                    Some(cur_i),
                ));
            }
        }

        if let Some(physics_tilemap) = &mut physics_tilemap {
            physics_tilemap.spawn_queue.extend(aabbs);
        } else {
            commands.entity(entity).insert(PhysicsTilemap {
                storage: Default::default(),
                spawn_queue: aabbs,
                data: ChunkedStorage::default(),
            });
        }

        commands.entity(entity).remove::<DataPhysicsTilemap>();
    }
}
