use bevy::{ecs::component::Component, math::UVec2, reflect::Reflect, utils::HashMap};

use crate::math::TileArea;

#[derive(Component, Clone, Copy, Reflect)]
pub struct PathTile {
    pub cost: u32,
}

#[derive(Component, Reflect)]
pub struct PathTilemap {
    pub(crate) tiles: HashMap<UVec2, PathTile>,
}

impl PathTilemap {
    pub fn new() -> Self {
        Self {
            tiles: HashMap::default(),
        }
    }

    pub fn get(&self, index: UVec2) -> Option<&PathTile> {
        self.tiles.get(&index)
    }

    pub fn get_mut(&mut self, index: UVec2) -> Option<&mut PathTile> {
        self.tiles.get_mut(&index)
    }

    pub fn set(&mut self, index: UVec2, new_tile: PathTile) {
        self.tiles.insert(index, new_tile);
    }

    /// Set path-finding data using a custom function.
    /// Before fill path, you need to fill the tiles first.
    /// Those empty indices will be ignored.
    pub fn fill_path_rect_custom(
        &mut self,
        area: TileArea,
        mut path_tile: impl FnMut(UVec2) -> PathTile,
    ) {
        for y in area.origin.y..=area.dest.y {
            for x in area.origin.x..=area.dest.x {
                let idx = UVec2::new(x, y);
                self.tiles.insert(idx, path_tile(idx));
            }
        }
    }

    /// Fill path-finding data using `PathTile`.
    /// Before fill path, you need to fill the tiles first.
    /// Those empty indices will be ignored.
    pub fn fill_path_rect(&mut self, area: TileArea, path_tile: &PathTile) {
        for y in area.origin.y..=area.dest.y {
            for x in area.origin.x..=area.dest.x {
                self.tiles.insert(UVec2::new(x, y), path_tile.clone());
            }
        }
    }
}
