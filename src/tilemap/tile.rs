use bevy::{
    ecs::system::{ParallelCommands, Query},
    math::IVec2,
    prelude::{Component, Entity},
    reflect::Reflect,
    render::{color::Color, render_resource::ShaderType},
};

use super::{buffers::Tiles, map::TilemapStorage};

/// A tile layer. This is the logical representation of a tile layer.
/// Not all the layers you added to a tile will be taken into consideration
/// when rendering. Only the top 4 layers will be rendered.
#[derive(Debug, Clone, Copy, Reflect)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub struct TileLayer {
    pub texture_index: i32,
    #[reflect(ignore)]
    pub flip: TileFlip,
}

impl Default for TileLayer {
    /// The default empty layer.
    fn default() -> Self {
        Self {
            texture_index: -1,
            flip: Default::default(),
        }
    }
}

impl TileLayer {
    #[inline]
    pub fn no_flip(texture_index: i32) -> Self {
        Self {
            texture_index,
            flip: TileFlip::NONE,
        }
    }

    #[inline]
    pub fn flip_h(texture_index: i32) -> Self {
        Self {
            texture_index,
            flip: TileFlip::HORIZONTAL,
        }
    }

    #[inline]
    pub fn flip_v(texture_index: i32) -> Self {
        Self {
            texture_index,
            flip: TileFlip::VERTICAL,
        }
    }

    #[inline]
    pub fn flip_both(texture_index: i32) -> Self {
        Self {
            texture_index,
            flip: TileFlip::BOTH,
        }
    }
}

/// The position of a tile layer.
#[derive(Debug, Clone, Copy, Reflect)]
pub enum TileLayerPosition {
    Top,
    Bottom,
    Index(usize),
}

#[derive(Clone, Reflect)]
pub struct LayerUpdater {
    pub position: TileLayerPosition,
    pub layer: TileLayer,
}

/// A tile layer updater. This is is useful when you want to change some properties
/// while not changing the whole tile.
#[derive(Default, Component, Clone, Reflect)]
pub struct TileUpdater {
    pub layer: Option<LayerUpdater>,
    pub tint: Option<Color>,
}

bitflags::bitflags! {
    /// The flip of a tile.
    #[derive(Debug, Clone, Copy)]
    #[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
    pub struct TileFlip: u32 {
        const NONE = 0b00;
        const HORIZONTAL = 0b01;
        const VERTICAL = 0b10;
        const BOTH = 0b11;
    }
}

impl Default for TileFlip {
    fn default() -> Self {
        Self::NONE
    }
}

/// A tile builder. This is used to create a tile.
#[derive(Debug, Clone, Reflect)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub struct TileBuilder {
    pub(crate) texture: TileTexture,
    pub(crate) tint: Color,
}

impl Tiles for TileBuilder {}

impl TileBuilder {
    /// Create a new tile builder.
    pub fn new() -> Self {
        Self {
            texture: TileTexture::Static(Vec::new()),
            tint: Color::WHITE,
        }
    }

    /// Set the tint of the entire tile. Default is white.
    pub fn with_tint(mut self, tint: Color) -> Self {
        self.tint = tint;
        self
    }

    /// Set the specific layer of the tile.
    ///
    /// You don't need to worry about the index of the layer. If the index is greater than the current
    /// layer count, the layer vector will be automatically resized.
    ///
    /// Notice that you can only add one animation to a tile or multiple static layers.
    pub fn with_layer(mut self, index: usize, layer: TileLayer) -> Self {
        if let TileTexture::Static(ref mut tex) = self.texture {
            if tex.len() <= index {
                tex.resize(index + 1, TileLayer::default());
            }
            tex[index] = layer;
        }
        self
    }

    /// Set the animation of the tile.
    ///
    /// Notice that you can only add one animation to a tile or multiple static layers.
    pub fn with_animation(mut self, animation: TileAnimation) -> Self {
        self.texture = TileTexture::Animated(animation);
        self
    }

    pub(crate) fn build_component(
        &self,
        index: IVec2,
        storage: &TilemapStorage,
        tilemap: Entity,
    ) -> Tile {
        let indices = storage.storage.transform_index(index);
        Tile {
            tilemap_id: tilemap,
            chunk_index: indices.0,
            in_chunk_index: indices.1,
            index,
            texture: self.texture.clone(),
            tint: self.tint,
        }
    }
}

/// A tile animation. This is actually information about the position of the animation
/// in the tilemap animation buffer. So it's cheap to clone.
#[derive(ShaderType, Debug, Clone, Copy, Reflect)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub struct TileAnimation {
    pub(crate) start: u32,
    pub(crate) length: u32,
    pub(crate) fps: u32,
}

/// A raw tile animation. This is contains the full information of a tile animation.
#[derive(Debug, Clone, Reflect)]
pub struct RawTileAnimation {
    pub sequence: Vec<u32>,
    pub fps: u32,
}

/// A tile texture. This is either a static texture or an animation.
#[derive(Debug, Clone, Reflect)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub enum TileTexture {
    Static(Vec<TileLayer>),
    Animated(TileAnimation),
}

/// The component of a tile.
#[derive(Component, Clone, Debug, Reflect)]
pub struct Tile {
    pub tilemap_id: Entity,
    pub chunk_index: IVec2,
    pub in_chunk_index: usize,
    pub index: IVec2,
    pub texture: TileTexture,
    pub tint: Color,
}

impl Tiles for Tile {}

impl Into<TileBuilder> for Tile {
    fn into(self) -> TileBuilder {
        TileBuilder {
            texture: self.texture,
            tint: self.tint,
        }
    }
}

pub fn tile_updater(
    commands: ParallelCommands,
    mut tiles_query: Query<(Entity, &mut Tile, &TileUpdater)>,
) {
    tiles_query
        .par_iter_mut()
        .for_each(|(entity, mut tile, updater)| {
            if let Some(layer) = &updater.layer {
                if let TileTexture::Static(ref mut tex) = tile.texture {
                    match layer.position {
                        TileLayerPosition::Top => {
                            tex.push(layer.layer);
                        }
                        TileLayerPosition::Bottom => {
                            tex.insert(0, layer.layer);
                        }
                        TileLayerPosition::Index(i) => {
                            if i >= tex.len() {
                                tex.resize(i + 1, TileLayer::default());
                            }
                            tex[i] = layer.layer;
                        }
                    }
                }
            }
            if let Some(color) = updater.tint {
                tile.tint = color;
            }
            commands.command_scope(|mut c| {
                c.entity(entity).remove::<TileUpdater>();
            });
        });
}
