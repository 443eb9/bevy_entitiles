use bevy::{
    hierarchy::BuildChildren,
    math::IVec2,
    prelude::{Commands, Component, Entity, UVec2, Vec4},
    reflect::Reflect,
};

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

#[derive(Debug, Clone, Reflect)]
pub struct TileBuilder {
    pub(crate) texture: TileTexture,
    pub(crate) color: Vec4,
}

impl TileBuilder {
    /// Create a new tile builder.
    pub fn new() -> Self {
        Self {
            texture: TileTexture::Static(Vec::new()),
            color: Vec4::ONE,
        }
    }

    pub fn with_color(mut self, color: Vec4) -> Self {
        self.color = color;
        self
    }

    pub fn with_layer(mut self, index: usize, layer: TileLayer) -> Self {
        if let TileTexture::Static(ref mut tex) = self.texture {
            if tex.len() <= index {
                tex.resize(index + 1, TileLayer::new());
            }
            tex[index] = layer;
        }
        self
    }

    pub fn with_animation(mut self, animation: u32) -> Self {
        self.texture = TileTexture::Animated(animation);
        self
    }

    pub(crate) fn build(&self, commands: &mut Commands, index: IVec2, tilemap: &Tilemap) -> Entity {
        let tile = self.build_component(index, tilemap);

        let mut tile_entity = commands.spawn_empty();
        tile_entity.insert(tile);
        let tile_entity = tile_entity.id();
        commands.entity(tilemap.id).add_child(tile_entity);
        tile_entity
    }

    pub(crate) fn build_component(&self, index: IVec2, tilemap: &Tilemap) -> Tile {
        let indices = tilemap.transform_index(index);
        Tile {
            tilemap_id: tilemap.id,
            chunk_index: indices.0,
            in_chunk_index: indices.1,
            index,
            texture: self.texture.clone(),
            color: self.color,
        }
    }
}

#[derive(Debug, Clone, Reflect)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub enum TileTexture {
    Static(Vec<TileLayer>),
    Animated(u32),
}

#[derive(Component, Clone, Debug, Reflect)]
pub struct Tile {
    pub tilemap_id: Entity,
    pub chunk_index: IVec2,
    pub in_chunk_index: UVec2,
    pub index: IVec2,
    pub texture: TileTexture,
    pub color: Vec4,
}

#[derive(Debug, Clone, Reflect)]
pub struct TileBuffer {
    pub(crate) size: UVec2,
    pub(crate) tiles: Vec<Option<TileBuilder>>,
}

impl TileBuffer {
    pub fn new(size: UVec2) -> Self {
        Self {
            size,
            tiles: vec![None; (size.x * size.y) as usize],
        }
    }

    pub fn size(&self) -> UVec2 {
        self.size
    }

    pub fn set(&mut self, index: UVec2, tile: TileBuilder) {
        self.tiles[(index.y * self.size.x + index.x) as usize] = Some(tile);
    }

    pub fn get(&self, index: UVec2) -> Option<&TileBuilder> {
        self.tiles
            .get((index.y * self.size.x + index.x) as usize)
            .and_then(|t| t.as_ref())
    }

    pub fn get_mut(&mut self, index: UVec2) -> Option<&mut TileBuilder> {
        self.tiles
            .get_mut((index.y * self.size.x + index.x) as usize)
            .and_then(|t| t.as_mut())
    }
}
