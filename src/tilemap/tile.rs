use bevy::prelude::{Commands, Component, Entity, Handle, Image, UVec2, Vec4};

use crate::render::texture::TilemapTextureDescriptor;

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

#[derive(Component, Clone)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub struct TileAnimation {
    pub sequence: Vec<u32>,
    pub fps: f32,
    pub is_loop: bool,
}

#[derive(Clone, Default)]
pub struct TilemapTexture {
    pub(crate) texture: Handle<Image>,
    pub(crate) desc: TilemapTextureDescriptor,
}

impl TilemapTexture {
    pub fn clone_weak(&self) -> Handle<Image> {
        self.texture.clone_weak()
    }

    pub fn desc(&self) -> &TilemapTextureDescriptor {
        &self.desc
    }

    pub fn handle(&self) -> &Handle<Image> {
        &self.texture
    }
}

#[derive(Clone)]
pub struct TileBuilder {
    pub(crate) texture_index: u32,
    pub(crate) anim: Option<TileAnimation>,
    pub(crate) color: Vec4,
}

impl TileBuilder {
    /// Create a new tile builder.
    pub fn new(texture_index: u32) -> Self {
        Self {
            texture_index,
            anim: None,
            color: Vec4::ONE,
        }
    }

    pub fn from_texture_index(texture_index: u32) -> Self {
        Self {
            texture_index,
            anim: None,
            color: Vec4::ONE,
        }
    }

    #[cfg(feature = "serializing")]
    pub fn from_serialized_tile(serialized_tile: &crate::serializing::SerializedTile) -> Self {
        Self {
            texture_index: serialized_tile.texture_index,
            anim: serialized_tile.anim.clone(),
            color: serialized_tile.color,
        }
    }

    pub fn with_color(mut self, color: Vec4) -> Self {
        self.color = color;
        self
    }

    pub fn with_animation(mut self, anim: TileAnimation) -> Self {
        self.anim = Some(anim);
        self
    }

    /// Build the tile and spawn it.
    ///
    /// # Note
    /// DO NOT call this method manually unless you really need to.
    ///
    /// Use `Tilemap::set` or `Tilemap::fill_xxx` instead.
    pub fn build(&self, commands: &mut Commands, index: UVec2, tilemap: &Tilemap) -> Entity {
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
            texture_index: self.texture_index,
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
    pub texture_index: u32,
    pub color: Vec4,
}
