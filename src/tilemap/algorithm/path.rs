use bevy::{ecs::component::Component, math::IVec2, reflect::Reflect};

use crate::{math::TileArea, tilemap::storage::ChunkedStorage, DEFAULT_CHUNK_SIZE};

#[derive(Component, Debug, Clone, Copy, Reflect)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub struct PathTile {
    pub cost: u32,
}

#[derive(Component, Debug, Clone, Reflect)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub struct PathTilemap {
    pub(crate) storage: ChunkedStorage<PathTile>,
}

impl PathTilemap {
    pub fn new() -> Self {
        Self {
            storage: ChunkedStorage::new(DEFAULT_CHUNK_SIZE),
        }
    }

    pub fn new_with_chunk_size(chunk_size: u32) -> Self {
        Self {
            storage: ChunkedStorage::new(chunk_size),
        }
    }

    pub fn get(&self, index: IVec2) -> Option<&PathTile> {
        self.storage.get_elem(index)
    }

    pub fn get_mut(&mut self, index: IVec2) -> Option<&mut PathTile> {
        self.storage.get_elem_mut(index)
    }

    pub fn set(&mut self, index: IVec2, new_tile: Option<PathTile>) {
        self.storage.set_elem(index, new_tile)
    }

    /// Set path-finding data using a custom function.
    pub fn fill_path_rect_custom(
        &mut self,
        area: TileArea,
        path_tile: impl Fn(IVec2) -> Option<PathTile>,
    ) {
        for y in area.origin.y..=area.dest.y {
            for x in area.origin.x..=area.dest.x {
                let idx = IVec2::new(x, y);
                self.set(idx, path_tile(idx));
            }
        }
    }

    /// Fill path-finding data using `PathTile`.
    pub fn fill_path_rect(&mut self, area: TileArea, path_tile: &PathTile) {
        for y in area.origin.y..=area.dest.y {
            for x in area.origin.x..=area.dest.x {
                self.set(IVec2 { x, y }, Some(*path_tile));
            }
        }
    }
}
