use bevy::reflect::Reflect;
use serde::{Deserialize, Serialize};

use crate::tilemap::buffers::TileBuilderBuffer;

#[cfg(feature = "algorithm")]
use crate::tilemap::buffers::PathTilesBuffer;
#[cfg(feature = "physics")]
use crate::tilemap::buffers::PhysicsTilesBuffer;

#[derive(Serialize, Deserialize, Debug, Clone, Reflect)]
pub struct TilemapPattern {
    pub(crate) label: Option<String>,
    pub(crate) tiles: TileBuilderBuffer,
    #[cfg(feature = "algorithm")]
    pub(crate) path_tiles: PathTilesBuffer,
    #[cfg(feature = "physics")]
    pub(crate) physics_tiles: PhysicsTilesBuffer,
}

impl TilemapPattern {
    pub fn new(label: Option<String>) -> Self {
        Self {
            label,
            tiles: TileBuilderBuffer::new(),
            #[cfg(feature = "algorithm")]
            path_tiles: PathTilesBuffer::new(),
            #[cfg(feature = "physics")]
            physics_tiles: PhysicsTilesBuffer::new(),
        }
    }
}
