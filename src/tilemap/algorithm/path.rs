use bevy::{ecs::component::Component, math::UVec2, reflect::Reflect};

use crate::math::TileArea;

#[derive(Component, Debug, Clone, Copy, Reflect)]
pub struct PathTile {
    pub cost: u32,
}

#[derive(Component, Debug, Clone, Reflect)]
pub struct PathTilemap {
    pub(crate) size: UVec2,
    pub(crate) tiles: Vec<Option<PathTile>>,
}

impl PathTilemap {
    pub fn new(size: UVec2) -> Self {
        Self {
            size,
            tiles: vec![None; (size.x * size.y) as usize],
        }
    }

    pub fn get(&self, index: UVec2) -> Option<&PathTile> {
        self.tiles
            .get(self.transform_index(index))
            .and_then(|t| t.as_ref())
    }

    pub fn get_mut(&mut self, index: UVec2) -> Option<&mut PathTile> {
        let index = self.transform_index(index);
        self.tiles.get_mut(index).and_then(|t| t.as_mut())
    }

    pub fn set(&mut self, index: UVec2, new_tile: PathTile) {
        let index = self.transform_index(index);
        self.tiles[index] = Some(new_tile);
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
                self.set(UVec2 { x, y }, *path_tile);
            }
        }
    }

    #[inline]
    pub fn transform_index(&self, index: UVec2) -> usize {
        (index.y * self.size.x + index.x) as usize
    }
}
