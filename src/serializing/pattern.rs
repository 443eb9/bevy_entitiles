use bevy::{ecs::system::Commands, math::UVec2, reflect::Reflect};
use serde::{Deserialize, Serialize};

use crate::tilemap::{map::Tilemap, tile::TileBuffer};

use super::{SerializedPathTile, SerializedTile};

#[derive(Serialize, Deserialize, Debug, Clone, Reflect)]
pub struct TilemapPattern {
    pub size: UVec2,
    pub tiles: Vec<Option<SerializedTile>>,
    #[cfg(feature = "algorithm")]
    pub path_tiles: Option<Vec<Option<SerializedPathTile>>>,
}

impl TilemapPattern {
    pub fn get(&self, index: UVec2) -> Option<&SerializedTile> {
        self.tiles[(index.y * self.size.x + index.x) as usize].as_ref()
    }

    pub fn apply(&self, commands: &mut Commands, origin: UVec2, target: &mut Tilemap) {
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
                            Some(tile.to_tile_builder())
                        } else {
                            None
                        }
                    })
                    .collect(),
            },
        );
    }
}
