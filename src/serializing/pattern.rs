use crate::{prelude::TilemapAnimations, tilemap::buffers::TileBuffer};
use bevy::reflect::Reflect;
use serde::{Deserialize, Serialize};

use crate::tilemap::buffers::TileBuilderBuffer;

#[cfg(feature = "algorithm")]
use crate::tilemap::buffers::PathTileBuffer;

#[cfg(feature = "physics")]
use crate::tilemap::physics::SerializablePhysicsSource;

#[derive(Serialize, Deserialize, Debug, Clone, Reflect)]
pub struct TilemapPattern {
    pub label: Option<String>,
    pub tiles: TileBuilderBuffer,
    pub animations: TilemapAnimations,
    #[cfg(feature = "algorithm")]
    pub path_tiles: PathTileBuffer,
    #[cfg(feature = "physics")]
    pub physics_tiles: SerializablePhysicsSource,
}

impl TilemapPattern {
    pub fn new(label: Option<String>) -> Self {
        TilemapPattern {
            label,
            tiles: TileBuffer::new(),
            animations: TilemapAnimations::default(),
            #[cfg(feature = "algorithm")]
            path_tiles: TileBuffer::new(),
            #[cfg(feature = "physics")]
            physics_tiles: SerializablePhysicsSource::Buffer(TileBuffer::new()),
        }
    }
}
