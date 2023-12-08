use bevy::prelude::{Commands, Component, Entity, UVec2, Vec4};

use super::map::Tilemap;

#[derive(Default, PartialEq, Eq, Hash, Clone, Copy, Debug)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub enum TileType {
    /// The most basic shape.
    #[default]
    Square,
    /// A diamond shape. It's like a square but rotated 45 degrees counterclockwise around the origin.
    /// But the coordinate system is the same as `Square`.
    IsometricDiamond,
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
    pub(crate) texture_index: Vec<Option<u32>>,
    pub(crate) top_layer: usize,
    pub(crate) anim: Option<AnimatedTile>,
    pub(crate) color: Vec4,
}

impl TileBuilder {
    /// Create a new tile builder.
    pub fn new(texture_index: u32) -> Self {
        Self {
            texture_index: vec![Some(texture_index)],
            anim: None,
            top_layer: 0,
            color: Vec4::ONE,
        }
    }

    #[cfg(feature = "serializing")]
    pub fn from_serialized_tile(serialized_tile: &crate::serializing::SerializedTile) -> Self {
        Self {
            texture_index: serialized_tile.texture_index.clone(),
            top_layer: serialized_tile.top_layer,
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

    pub fn with_layer(mut self, layer: usize, texture_index: u32) -> Self {
        if let Some(anim) = self.anim.as_mut() {
            anim.layer = layer;
        } else if layer >= self.texture_index.len() {
            let n = layer as i32 - self.texture_index.len() as i32;
            if n > 0 {
                self.texture_index.extend(vec![None; n as usize]);
            }

            self.top_layer = layer;
            self.texture_index.push(Some(texture_index));
        } else {
            self.top_layer = self.top_layer.max(layer);
            self.texture_index[layer] = Some(texture_index);
        }

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
            texture_index: self.texture_index.clone(),
            top_layer: 0,
            color: self.color,
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
    pub texture_index: Vec<Option<u32>>,
    pub top_layer: usize,
    pub color: Vec4,
}

#[derive(Component, Clone)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub struct AnimatedTile {
    pub layer: usize,
    pub sequence: Vec<u32>,
    pub fps: f32,
    pub is_loop: bool,
}
