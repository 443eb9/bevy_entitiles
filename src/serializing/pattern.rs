use bevy::{
    ecs::system::Commands,
    math::{IVec2, UVec2},
    reflect::Reflect,
};
use serde::{Deserialize, Serialize};

use crate::tilemap::{map::Tilemap, tile::TileBuffer};

use super::SerializedTile;

#[derive(Serialize, Deserialize, Debug, Clone, Reflect)]
pub struct TilemapPattern {
    pub label: Option<String>,
    pub size: UVec2,
    pub tiles: Vec<Option<SerializedTile>>,
    #[cfg(feature = "algorithm")]
    pub path_tiles: Option<bevy::utils::HashMap<IVec2, super::SerializedPathTile>>,
}

impl TilemapPattern {
    pub fn new(label: Option<String>, size: UVec2) -> Self {
        Self {
            label,
            size,
            tiles: vec![None; (size.x * size.y) as usize],
            #[cfg(feature = "algorithm")]
            path_tiles: None,
        }
    }

    pub fn get(&self, index: UVec2) -> Option<&SerializedTile> {
        self.tiles[(index.y * self.size.x + index.x) as usize].as_ref()
    }

    pub fn get_mut(&mut self, index: UVec2) -> Option<&mut SerializedTile> {
        self.tiles[(index.y * self.size.x + index.x) as usize].as_mut()
    }

    pub fn set(&mut self, index: UVec2, tile: Option<SerializedTile>) {
        self.tiles[(index.y * self.size.x + index.x) as usize] = tile;
    }

    pub fn is_index_oobu(&self, index: UVec2) -> bool {
        index.x >= self.size.x || index.y >= self.size.y
    }

    pub fn is_index_oobi(&self, index: IVec2) -> bool {
        index.x >= self.size.x as i32 || index.y >= self.size.y as i32 || index.x < 0 || index.y < 0
    }

    pub fn apply(&self, commands: &mut Commands, origin: IVec2, target: &mut Tilemap) {
        target.fill_with_buffer(
            commands,
            origin,
            TileBuffer {
                size: self.size,
                tiles: self
                    .tiles
                    .clone()
                    .into_iter()
                    .map(|t| {
                        if let Some(tile) = t {
                            Some(tile.into())
                        } else {
                            None
                        }
                    })
                    .collect(),
            },
        );
    }
}
