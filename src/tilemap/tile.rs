use bevy::{
    hierarchy::BuildChildren,
    prelude::{Commands, Component, Entity, UVec2, Vec4},
    reflect::Reflect,
};

use crate::math::extension::DivToCeil;

use super::{layer::TileLayer, map::Tilemap};

/// Defines the shape of tiles in a tilemap.
/// Check the `Coordinate Systems` chapter in README.md to see the details.
#[derive(Default, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub enum TileType {
    #[default]
    Square,
    Isometric,
    Hexagonal(u32),
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, Reflect)]
pub enum TileFlip {
    None = 0b00,
    Horizontal = 0b01,
    Vertical = 0b10,
    Both = 0b11,
}

impl From<u32> for TileFlip {
    fn from(value: u32) -> Self {
        match value {
            0b00 => Self::None,
            0b01 => Self::Horizontal,
            0b10 => Self::Vertical,
            0b11 => Self::Both,
            _ => panic!("Invalid flip value! {}", value),
        }
    }
}

#[derive(Clone)]
pub struct TileBuilder {
    pub(crate) layers: Vec<TileLayer>,
    pub(crate) anim: Option<AnimatedTile>,
    pub(crate) color: Vec4,
}

impl TileBuilder {
    /// Create a new tile builder.
    pub fn new() -> Self {
        Self {
            layers: Vec::new(),
            anim: None,
            color: Vec4::ONE,
        }
    }

    #[cfg(feature = "serializing")]
    pub fn from_serialized_tile(serialized_tile: &crate::serializing::SerializedTile) -> Self {
        Self {
            layers: serialized_tile.layers.clone(),
            anim: serialized_tile.anim.clone(),
            color: serialized_tile.color,
        }
    }

    pub fn with_color(mut self, color: Vec4) -> Self {
        self.color = color;
        self
    }

    pub fn with_animation(mut self, anim: AnimatedTile) -> Self {
        self.anim = Some(anim);
        self
    }

    pub fn with_layer(mut self, index: usize, layer: TileLayer) -> Self {
        if self.layers.len() <= index {
            self.layers.resize(index + 1, TileLayer::new());
        }
        self.layers[index] = layer;
        self
    }

    pub(crate) fn build(&self, commands: &mut Commands, index: UVec2, tilemap: &Tilemap) -> Entity {
        let (tile, anim) = self.build_component(index, tilemap);

        let mut tile_entity = commands.spawn_empty();
        tile_entity.insert(tile);
        if let Some(anim) = anim {
            tile_entity.insert(anim);
        }

        let tile_entity = tile_entity.id();
        commands.entity(tilemap.id).add_child(tile_entity);
        tile_entity
    }

    pub(crate) fn build_component(
        &self,
        index: UVec2,
        tilemap: &Tilemap,
    ) -> (Tile, Option<AnimatedTile>) {
        let chunk_index = index / tilemap.render_chunk_size;
        let storage_size = tilemap
            .size
            .div_to_ceil(UVec2::splat(tilemap.render_chunk_size));
        let chunk_index_vec = chunk_index.y * storage_size.x + chunk_index.x;

        (
            Tile {
                render_chunk_index: chunk_index_vec as usize,
                tilemap_id: tilemap.id,
                index,
                layers: self.layers.clone(),
                color: self.color,
            },
            self.anim.clone(),
        )
    }
}

#[derive(Component, Clone, Debug, Reflect)]
pub struct Tile {
    pub tilemap_id: Entity,
    pub render_chunk_index: usize,
    pub index: UVec2,
    pub layers: Vec<TileLayer>,
    pub color: Vec4,
}

#[derive(Component, Clone, Reflect)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub struct AnimatedTile {
    pub sequence_index: usize,
}
