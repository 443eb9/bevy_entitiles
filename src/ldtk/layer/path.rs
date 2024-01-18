use bevy::{ecs::system::Resource, math::IVec2, reflect::Reflect, utils::HashMap};

use crate::{
    ldtk::json::{definitions::LayerType, level::LayerInstance},
    tilemap::algorithm::path::PathTile,
};

#[derive(Debug, Resource, Clone, Reflect)]
pub struct LdtkPathLayer {
    pub identifier: String,
    pub parent: String,
    pub cost_mapper: Option<HashMap<i32, u32>>,
}

pub fn analyze_path_layer(layer: &LayerInstance, path: &LdtkPathLayer) -> HashMap<IVec2, PathTile> {
    if layer.ty != LayerType::IntGrid {
        panic!(
            "The path layer {:?} is not an IntGrid layer!",
            layer.identifier
        );
    }

    let size = IVec2::new(layer.c_wid, layer.c_hei);
    let mut tiles = HashMap::with_capacity((size.x * size.y) as usize);
    let grid = &layer.int_grid_csv;
    let cost_mapper = path.cost_mapper.clone().unwrap_or_default();

    for y in 0..size.y {
        for x in 0..size.x {
            tiles.insert(
                IVec2 { x, y },
                PathTile {
                    cost: *cost_mapper
                        .get(&grid[(y * size.x + x) as usize])
                        .unwrap_or(&(grid[(y * size.x + x) as usize] as u32)),
                },
            );
        }
    }

    tiles
}
