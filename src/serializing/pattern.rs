use bevy::reflect::Reflect;
use serde::{Deserialize, Serialize};

use crate::tilemap::buffers::TileBuilderBuffer;

#[cfg(feature = "algorithm")]
use crate::tilemap::buffers::PathTileBuffer;
#[cfg(feature = "physics")]
use crate::tilemap::buffers::PhysicsTileBuffer;

#[derive(Serialize, Deserialize, Debug, Clone, Reflect)]
pub struct TilemapPattern {
    pub(crate) label: Option<String>,
    pub(crate) tiles: TileBuilderBuffer,
    #[cfg(feature = "algorithm")]
    pub(crate) path_tiles: PathTileBuffer,
    #[cfg(feature = "physics")]
    pub(crate) physics_tiles: PhysicsTileBuffer,
}

impl TilemapPattern {
    pub fn new(label: Option<String>) -> Self {
        Self {
            label,
            tiles: TileBuilderBuffer::new(),
            #[cfg(feature = "algorithm")]
            path_tiles: PathTileBuffer::new(),
            #[cfg(feature = "physics")]
            physics_tiles: PhysicsTileBuffer::new(),
        }
    }
}
