use bevy::{ecs::component::Component, math::IVec2, reflect::Reflect, utils::HashMap};

use crate::math::TileArea;

#[derive(Component, Debug, Clone, Copy, Reflect)]
pub struct PathTile {
    pub cost: u32,
}

#[derive(Component, Debug, Clone, Reflect)]
pub struct PathTilemap {
    pub(crate) storage: HashMap<IVec2, PathTile>,
}

impl PathTilemap {
    pub fn new() -> Self {
        Self {
            storage: HashMap::new(),
        }
    }

    pub fn get(&self, index: IVec2) -> Option<&PathTile> {
        self.storage.get(&index)
    }

    pub fn get_mut(&mut self, index: IVec2) -> Option<&mut PathTile> {
        self.storage.get_mut(&index)
    }

    pub fn set(&mut self, index: IVec2, new_tile: Option<PathTile>) {
        if let Some(tile) = new_tile {
            self.storage.insert(index, tile);
        } else {
            self.storage.remove(&index);
        }
    }

    /// Set path-finding data using a custom function.
    /// Before fill path, you need to fill the tiles first.
    /// Those empty indices will be ignored.
    pub fn fill_path_rect_custom(
        &mut self,
        area: TileArea,
        mut path_tile: impl FnMut(IVec2) -> Option<PathTile>,
    ) {
        for y in area.origin.y..=area.dest.y {
            for x in area.origin.x..=area.dest.x {
                let idx = IVec2::new(x, y);
                self.set(idx, path_tile(idx));
            }
        }
    }

    /// Fill path-finding data using `PathTile`.
    /// Before fill path, you need to fill the tiles first.
    /// Those empty indices will be ignored.
    pub fn fill_path_rect(&mut self, area: TileArea, path_tile: &PathTile) {
        for y in area.origin.y..=area.dest.y {
            for x in area.origin.x..=area.dest.x {
                self.set(IVec2 { x, y }, Some(*path_tile));
            }
        }
    }
}
