use bevy::{
    ecs::system::Query,
    hierarchy::BuildChildren,
    math::{IVec4, UVec4},
    prelude::{Commands, Component, Entity, UVec2, Vec4}, reflect::Reflect,
};

use crate::MAX_LAYER_COUNT;

use super::map::Tilemap;

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
    pub(crate) texture_indices: IVec4,
    pub(crate) anim: Option<AnimatedTile>,
    pub(crate) color: Vec4,
    pub(crate) flip: UVec4,
}

impl TileBuilder {
    /// Create a new tile builder.
    pub fn new() -> Self {
        Self {
            texture_indices: IVec4::NEG_ONE,
            anim: None,
            color: Vec4::ONE,
            flip: UVec4::ZERO,
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

    pub fn with_flip(mut self, layer: usize, flip: TileFlip) -> Self {
        self.flip[layer] |= flip as u32;
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
            color: self.color,
            flip: self.flip,
        });
        if let Some(anim) = &self.anim {
            tile.insert(anim.clone());
        }

        let tile_entity = tile.id();
        commands.entity(tilemap.id).add_child(tile_entity);
        tile_entity
    }
}

#[derive(Component, Clone, Debug, Reflect)]
pub struct Tile {
    pub tilemap_id: Entity,
    pub render_chunk_index: usize,
    pub index: UVec2,
    pub texture_indices: IVec4,
    pub color: Vec4,
    pub flip: UVec4,
}

#[derive(Component, Clone, Reflect)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub struct AnimatedTile {
    pub sequence_index: usize,
}

#[derive(Default, Component, Clone, Copy, Reflect)]
pub struct TileUpdater {
    pub texture_index: Option<(usize, u32)>,
    pub color: Option<Vec4>,
    pub flip: Option<(usize, u32)>,
}

pub fn tile_updater(mut tiles_query: Query<(&mut Tile, &TileUpdater)>) {
    tiles_query.par_iter_mut().for_each(|(mut tile, updater)| {
        if let Some((layer, texture_index)) = updater.texture_index {
            tile.texture_indices[layer] = texture_index as i32;
        }
        if let Some(color) = updater.color {
            tile.color = color;
        }
        if let Some((layer, flip)) = updater.flip {
            tile.flip[layer] = flip;
        }
    });
}
