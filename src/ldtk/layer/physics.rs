use bevy::{math::UVec2, reflect::Reflect, utils::HashMap};

use crate::prelude::json::{level::LayerInstance, definitions::LayerType};

#[derive(Debug, Clone, Reflect)]
pub struct LdtkPhysicsLayer {
    pub identifier: String,
    pub air_value: i32,
    pub parent: String,
    pub frictions: Option<HashMap<i32, f32>>,
}

pub fn analyze_physics_layer(
    layer: &LayerInstance,
    physics: &LdtkPhysicsLayer,
) -> Vec<(i32, (UVec2, UVec2))> {
    if layer.ty != LayerType::IntGrid {
        panic!(
            "The physics layer {:?} is not an IntGrid layer!",
            layer.identifier
        );
    }

    parse_grid(
        layer.int_grid_csv.clone(),
        UVec2::new(layer.c_wid as u32, layer.c_hei as u32),
        physics.air_value,
    )
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
