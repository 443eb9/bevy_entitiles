use bevy::{math::UVec2, reflect::Reflect, utils::HashMap};

use crate::{
    prelude::json::{definitions::LayerType, level::LayerInstance},
    tilemap::algorithm::path::{PathTile, PathTilemap},
};

#[derive(Debug, Clone, Reflect)]
pub struct LdtkPathLayer {
    pub identifier: String,
    pub parent: String,
    pub cost_mapper: Option<HashMap<i32, u32>>,
}

pub fn analyze_path_layer(layer: &LayerInstance, path: &LdtkPathLayer) -> PathTilemap {
    if layer.ty != LayerType::IntGrid {
        panic!(
            "The path layer {:?} is not an IntGrid layer!",
            layer.identifier
        );
    }

    let size = UVec2::new(layer.c_wid as u32, layer.c_hei as u32);
    let mut tilemap = PathTilemap::new(size);
    let grid = &layer.int_grid_csv;
    let cost_mapper = path.cost_mapper.clone().unwrap_or_default();

    for y in 0..size.y {
        for x in 0..size.x {
            tilemap.set(
                UVec2 { x, y },
                PathTile {
                    cost: *cost_mapper
                        .get(&grid[(y * size.x + x) as usize])
                        .unwrap_or(&(grid[(y * size.x + x) as usize] as u32)),
                },
            )
        }
    }

    tilemap
}
