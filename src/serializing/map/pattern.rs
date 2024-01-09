use bevy::reflect::Reflect;
use serde::{Deserialize, Serialize};

use crate::{math::aabb::IAabb2d, tilemap::buffers::TileBuilderBuffer};

#[cfg(feature = "algorithm")]
use crate::tilemap::buffers::PathTilesBuffer;

#[derive(Serialize, Deserialize, Debug, Clone, Reflect)]
pub struct TilemapPattern {
    pub(crate) label: Option<String>,
    pub(crate) tiles: TileBuilderBuffer,
    #[cfg(feature = "algorithm")]
    pub(crate) path_tiles: PathTilesBuffer,
    pub(crate) aabb: IAabb2d,
}

impl TilemapPattern {
    pub fn new(label: Option<String>) -> Self {
        Self {
            label,
            tiles: TileBuilderBuffer::new(),
            #[cfg(feature = "algorithm")]
            path_tiles: PathTilesBuffer::new(),
            aabb: IAabb2d::default(),
        }
    }
}
