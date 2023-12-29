use bevy::{
    ecs::{component::Component, system::Query},
    math::Vec4,
    reflect::Reflect,
};

use super::tile::{Tile, TileFlip, TileTexture};

#[derive(Debug, Default, Clone, Copy, Reflect)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub struct TileLayer {
    pub(crate) texture_index: i32,
    pub(crate) flip: u32,
}

impl TileLayer {
    pub fn new() -> Self {
        Self {
            texture_index: -1,
            flip: 0,
        }
    }

    pub fn with_texture_index(mut self, texture_index: u32) -> Self {
        self.texture_index = texture_index as i32;
        self
    }

    pub fn with_flip(mut self, flip: TileFlip) -> Self {
        self.flip |= flip as u32;
        self
    }

    #[allow(dead_code)]
    pub(crate) fn with_flip_raw(mut self, flip: u32) -> Self {
        self.flip = flip;
        self
    }
}

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

#[derive(Default, Component, Clone, Reflect)]
pub struct TileUpdater {
    pub layer: Option<LayerUpdater>,
    pub color: Option<Vec4>,
}

pub fn tile_updater(mut tiles_query: Query<(&mut Tile, &TileUpdater)>) {
    tiles_query.par_iter_mut().for_each(|(mut tile, updater)| {
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
                            tex.resize(i + 1, TileLayer::new());
                        }
                        tex[i] = layer.layer;
                    }
                }
            }
        }
        if let Some(color) = updater.color {
            tile.color = color;
        }
    });
}
