use bevy::{
    math::IVec4,
    prelude::{Commands, Component, Entity, UVec2, Vec4},
};

use crate::MAX_LAYER_COUNT;

use super::map::Tilemap;

/// Defines the shape of tiles in a tilemap.
/// Check the `Coordinate Systems` chapter in README.md to see the details.
#[derive(Default, PartialEq, Eq, Hash, Clone, Copy, Debug)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub enum TileType {
    #[default]
    Square,
    Isometric,
    Hexagonal(u32),
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum TileFlip {
    Horizontal = 0b01,
    Vertical = 0b10,
    Both = 0b11,
}

#[derive(Clone)]
pub struct TileBuilder {
    pub(crate) texture_indices: IVec4,
    pub(crate) anim: Option<AnimatedTile>,
    pub(crate) color: Vec4,
    pub(crate) flip: u32,
}

impl TileBuilder {
    /// Create a new tile builder.
    pub fn new() -> Self {
        Self {
            texture_indices: IVec4::NEG_ONE,
            anim: None,
            color: Vec4::ONE,
            flip: 0,
        }
    }

    #[cfg(feature = "serializing")]
    pub fn from_serialized_tile(serialized_tile: &crate::serializing::SerializedTile) -> Self {
        Self {
            texture_indices: serialized_tile.texture_indices,
            anim: serialized_tile.anim.clone(),
            color: serialized_tile.color,
            flip: serialized_tile.flip,
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

    pub fn with_layer(mut self, layer: usize, texture_index: u32) -> Self {
        if self.anim.is_none() && layer < MAX_LAYER_COUNT {
            self.texture_indices[layer] = texture_index as i32;
        } else {
            panic!(
                "Trying to add a layer to an animated tile or the layer index is out of bounds!"
            );
        }

        self
    }

    pub fn with_flip(mut self, flip: TileFlip) -> Self {
        self.flip |= flip as u32;
        self
    }

    pub(crate) fn build(&self, commands: &mut Commands, index: UVec2, tilemap: &Tilemap) -> Entity {
        let render_chunk_index_2d = index / tilemap.render_chunk_size;
        let render_chunk_index = {
            if tilemap.size.x % tilemap.render_chunk_size == 0 {
                render_chunk_index_2d.y * (tilemap.size.x / tilemap.render_chunk_size)
                    + render_chunk_index_2d.x
            } else {
                render_chunk_index_2d.y * (tilemap.size.x / tilemap.render_chunk_size + 1)
                    + render_chunk_index_2d.x
            }
        } as usize;
        let mut tile = commands.spawn_empty();
        tile.insert(Tile {
            render_chunk_index,
            tilemap_id: tilemap.id,
            index,
            texture_indices: self.texture_indices,
            top_layer: 0,
            color: self.color,
            flip: self.flip,
        });
        if let Some(anim) = &self.anim {
            tile.insert(anim.clone());
        }
        tile.id()
    }
}

#[derive(Component, Clone, Debug)]
pub struct Tile {
    pub tilemap_id: Entity,
    pub render_chunk_index: usize,
    pub index: UVec2,
    pub texture_indices: IVec4,
    pub top_layer: usize,
    pub color: Vec4,
    pub flip: u32,
}

#[derive(Component, Clone)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub struct AnimatedTile {
    pub sequence_index: usize,
}
