use bevy::{
    ecs::{entity::Entity, system::Commands},
    hierarchy::BuildChildren,
    math::{IVec2, UVec2, Vec2},
    reflect::Reflect,
    transform::{components::Transform, TransformBundle},
    utils::HashMap,
};

use crate::{
    ldtk::json::{definitions::LayerType, level::LayerInstance},
    tilemap::{
        coordinates,
        map::{TilePivot, TilemapSlotSize, TilemapTransform, TilemapType},
    },
};

#[derive(Debug, Clone, Reflect)]
pub struct LdtkPhysicsLayer {
    pub identifier: String,
    pub air_value: i32,
    pub parent: String,
    pub frictions: Option<HashMap<i32, f32>>,
}

#[derive(Debug, Clone, Reflect)]
pub struct LdtkPhysicsAabbs {
    pub aabbs: Vec<(i32, (UVec2, UVec2))>,
}

impl LdtkPhysicsAabbs {
    pub fn generate_colliders(
        &self,
        commands: &mut Commands,
        tilemap: Entity,
        ty: &TilemapType,
        transform: &TilemapTransform,
        pivot: &TilePivot,
        slot_size: &TilemapSlotSize,
        frictions: Option<&HashMap<i32, f32>>,
        offset: Vec2,
    ) -> Vec<Entity> {
        self.aabbs
            .iter()
            .map(|(i, (min, max))| {
                let left_top = coordinates::index_to_world(
                    IVec2 {
                        x: min.x as i32,
                        y: -(min.y as i32),
                    },
                    ty,
                    transform,
                    pivot,
                    slot_size,
                );
                let right_btm = coordinates::index_to_world(
                    IVec2 {
                        x: (max.x + 1) as i32,
                        y: -(max.y as i32) - 1,
                    },
                    ty,
                    transform,
                    pivot,
                    slot_size,
                );
                (
                    i,
                    crate::math::aabb::Aabb2d {
                        min: Vec2 {
                            x: left_top.x,
                            y: right_btm.y,
                        },
                        max: Vec2 {
                            x: right_btm.x,
                            y: left_top.y,
                        },
                    },
                )
            })
            .map(|(i, aabb)| {
                let mut collider = commands.spawn(TransformBundle {
                    local: Transform::from_translation((aabb.center() + offset).extend(0.)),
                    ..Default::default()
                });
                collider.set_parent(tilemap);

                #[cfg(feature = "physics")]
                {
                    collider.insert((
                        bevy_xpbd_2d::components::Collider::cuboid(
                            aabb.width() - 0.02,
                            aabb.height() - 0.02,
                        ),
                        bevy_xpbd_2d::components::RigidBody::Static,
                    ));
                    if let Some(coe) = frictions.and_then(|f| f.get(i)) {
                        collider.insert(bevy_xpbd_2d::components::Friction {
                            dynamic_coefficient: *coe,
                            static_coefficient: *coe,
                            ..Default::default()
                        });
                    }
                }

                collider.id()
            })
            .collect()
    }
}

pub fn analyze_physics_layer(
    layer: &LayerInstance,
    physics: &LdtkPhysicsLayer,
) -> LdtkPhysicsAabbs {
    if layer.ty != LayerType::IntGrid {
        panic!(
            "The physics layer {:?} is not an IntGrid layer!",
            layer.identifier
        );
    }

    LdtkPhysicsAabbs {
        aabbs: parse_grid(
            layer.int_grid_csv.clone(),
            UVec2::new(layer.c_wid as u32, layer.c_hei as u32),
            physics.air_value,
        ),
    }
}

/// Returns a list of all the aabb colliders in the grid.
fn parse_grid(mut grid: Vec<i32>, size: UVec2, air_value: i32) -> Vec<(i32, (UVec2, UVec2))> {
    let mut aabbs = vec![];

    for y in 0..size.y {
        for x in 0..size.x {
            if get_value(&grid, x, y, size, air_value) == air_value {
                continue;
            }

            let cur_i = grid[(x + y * size.x) as usize];
            let cur = UVec2 { x, y };
            let mut d = UVec2 {
                x: if x == size.x - 1 { 0 } else { 1 },
                y: if y == size.y - 1 { 0 } else { 1 },
            };
            let mut dst = cur;
            while d.x != 0 || d.y != 0 {
                for t_x in cur.x..=dst.x {
                    if get_value(&grid, t_x, dst.y + d.y, size, air_value) != cur_i {
                        d.y = 0;
                        break;
                    }
                }

                for t_y in cur.y..=dst.y {
                    if get_value(&grid, dst.x + d.x, t_y, size, air_value) != cur_i {
                        d.x = 0;
                        break;
                    }
                }

                if d == UVec2::ONE
                    && get_value(&grid, dst.x + 1, dst.y + 1, size, air_value) != cur_i
                {
                    d.y = 0;
                }

                dst += d;
            }

            fill(&mut grid, cur, dst, size, air_value);
            aabbs.push((cur_i, (cur, dst)));
        }
    }
    aabbs
}

#[inline]
fn get_value(grid: &Vec<i32>, x: u32, y: u32, size: UVec2, air_value: i32) -> i32 {
    *grid.get((x + y * size.x) as usize).unwrap_or(&air_value)
}

fn fill(grid: &mut Vec<i32>, start: UVec2, end: UVec2, size: UVec2, value: i32) {
    for y in start.y..=end.y {
        for x in start.x..=end.x {
            grid[(x + y * size.x) as usize] = value;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parsing() {
        let grid = vec![
            1, 1, 1, 1, 1, 1, 1, //
            0, 0, 0, 1, 0, 1, 0, //
            0, 1, 1, 1, 1, 0, 0, //
            0, 1, 0, 1, 0, 1, 1, //
            0, 0, 0, 1, 0, 0, 0,
        ];
        let size = UVec2::new(7, 5);
        let aabbs = parse_grid(grid.clone(), size, 0);
        println!("{:?}", aabbs);
    }
}
