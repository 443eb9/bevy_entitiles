use crate::{
    prelude::TilemapAnimations,
    tilemap::{buffers::TileBuffer, map::TilemapTexture},
};
use bevy::{math::UVec2, reflect::Reflect};
use serde::{Deserialize, Serialize};

use crate::tilemap::buffers::TileBuilderBuffer;

#[cfg(feature = "algorithm")]
use crate::tilemap::buffers::PathTileBuffer;

#[cfg(feature = "physics")]
use crate::tilemap::physics::SerializablePhysicsSource;

/// A pattern of tiles.
///
/// This includes the tiles, animations, and other data.
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

/// A layer of patterns. This can be used when performing wfc.
#[derive(Clone, Reflect)]
pub struct PatternsLayer {
    pub(crate) pattern_size: UVec2,
    pub(crate) label: Option<String>,
    pub(crate) patterns: Vec<TilemapPattern>,
    pub(crate) texture: Option<TilemapTexture>,
}

impl PatternsLayer {
    pub fn new(
        label: Option<String>,
        pattern_size: UVec2,
        patterns: Vec<TilemapPattern>,
        texture: Option<TilemapTexture>,
    ) -> Self {
        patterns.iter().enumerate().for_each(|(i, p)| {
            assert_eq!(
                p.tiles.aabb.extent,
                pattern_size,
                "Pattern size mismatch! Pattern No.{}[label = {:?}]'s size is {}, \
                but the pattern_size is {}",
                i,
                p.label.clone().unwrap_or("None".to_string()),
                p.tiles.aabb.extent,
                pattern_size
            )
        });

        Self {
            label,
            pattern_size,
            patterns,
            texture,
        }
    }

    pub fn get(&self, index: usize) -> &TilemapPattern {
        &self.patterns[index]
    }

    pub fn iter(&self) -> impl Iterator<Item = &TilemapPattern> {
        self.patterns.iter()
    }
}

pub struct PatternElementSlice<'a> {
    pub index: usize,
    pub pattern_size: UVec2,
    pub element: Vec<(&'a TilemapPattern, &'a Option<TilemapTexture>)>,
}

/// The patterns packed in layers.
///
/// Layers are made of patterns with the same textures but different tiles.
#[derive(Clone, Reflect)]
pub struct PackedPatternLayers {
    pub(crate) pattern_size: UVec2,
    pub(crate) layers: Vec<PatternsLayer>,
}

impl PackedPatternLayers {
    pub fn new(pattern_size: UVec2, layers: Vec<PatternsLayer>) -> Self {
        layers.iter().enumerate().for_each(|(i, p)| {
            assert_eq!(
                p.pattern_size, pattern_size,
                "Pattern size mismatch! Layer No.{}'s size is {}, \
                but the pattern_size is {}",
                i, p.pattern_size, pattern_size
            )
        });

        Self {
            pattern_size,
            layers,
        }
    }

    /// Get the specific layer.
    pub fn get_layer(&self, index: usize) -> &PatternsLayer {
        &self.layers[index]
    }

    /// Get all the layers at the specific index.
    pub fn get_element(&self, index: usize) -> PatternElementSlice {
        PatternElementSlice {
            index,
            pattern_size: self.pattern_size,
            element: self
                .layers
                .iter()
                .map(|l| (l.get(index), &l.texture))
                .collect(),
        }
    }
}
