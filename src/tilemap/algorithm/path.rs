use bevy::{ecs::component::Component, math::IVec2, reflect::Reflect};

use crate::{
    math::TileArea,
    tilemap::{
        buffers::{PathTileBuffer, Tiles},
        chunking::storage::{ChunkedStorage, PathTileChunkedStorage},
    },
};

/// A tile for path-finding.
#[derive(Component, Debug, Clone, Copy, Reflect)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub struct PathTile {
    pub cost: u32,
}

impl Tiles for PathTile {}

/// A tilemap for path-finding.
#[derive(Debug, Clone, Reflect)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub struct PathTilemap {
    pub(crate) storage: PathTileChunkedStorage,
}

impl PathTilemap {
    /// Create a new path tilemap with default chunk size.
    /// 
    /// Use `new_with_chunk_size` to create a path tilemap with custom chunk size.
    pub fn new() -> Self {
        Self {
            storage: ChunkedStorage::default(),
        }
    }

    /// Create a new path tilemap with custom chunk size.
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

    pub fn set(&mut self, index: IVec2, tile: PathTile) {
        self.storage.set_elem(index, tile)
    }

    pub fn remove(&mut self, index: IVec2) -> Option<PathTile> {
        self.storage.remove_elem(index)
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
                if let Some(tile) = path_tile(idx) {
                    self.set(idx, tile);
                }
            }
        }
    }

    /// Fill path-finding data using the same `PathTile`.
    pub fn fill_path_rect(&mut self, area: TileArea, path_tile: PathTile) {
        for y in area.origin.y..=area.dest.y {
            for x in area.origin.x..=area.dest.x {
                self.set(IVec2 { x, y }, path_tile);
            }
        }
    }

    /// Fill path-finding data using a buffer.
    pub fn fill_with_buffer(&mut self, origin: IVec2, buffer: PathTileBuffer) {
        buffer.tiles.into_iter().for_each(|(index, tile)| {
            self.set(index + origin, tile);
        });
    }
}
