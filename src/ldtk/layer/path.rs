use bevy::{math::IVec2, reflect::Reflect, utils::HashMap};

use crate::{
    ldtk::json::{definitions::LayerType, level::LayerInstance},
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

    let size = IVec2::new(layer.c_wid, layer.c_hei);
    let mut tilemap = PathTilemap::new();
    let grid = &layer.int_grid_csv;
    let cost_mapper = path.cost_mapper.clone().unwrap_or_default();

    for y in 0..size.y {
        for x in 0..size.x {
            tilemap.set(
                IVec2 { x, y },
                Some(PathTile {
                    cost: *cost_mapper
                        .get(&grid[(y * size.x + x) as usize])
                        .unwrap_or(&(grid[(y * size.x + x) as usize] as u32)),
                }),
            )
        }
    }

    tilemap
}
