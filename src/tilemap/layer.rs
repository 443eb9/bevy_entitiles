use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        system::{ParallelCommands, Query},
    },
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

#[derive(Component, Reflect)]
pub struct LayerInserter {
    pub is_top: bool,
    pub layer: TileLayer,
}

pub fn layer_inserter(
    commands: ParallelCommands,
    mut tiles_query: Query<(Entity, &mut Tile, &LayerInserter)>,
) {
    tiles_query
        .par_iter_mut()
        .for_each(|(entity, mut tile, inserter)| {
            if inserter.is_top {
                if let TileTexture::Static(ref mut tex) = tile.texture {
                    tex.push(inserter.layer)
                }
            } else {
                if let TileTexture::Static(ref mut tex) = tile.texture {
                    tex.insert(0, inserter.layer)
                }
            }

            commands.command_scope(|mut c| {
                c.entity(entity).remove::<LayerInserter>();
            });
        });
}

#[derive(Default, Component, Clone, Copy, Reflect)]
pub struct LayerUpdater {
    pub index: Option<usize>,
    pub layer: TileLayer,
    pub color: Option<Vec4>,
}

pub fn layer_updater(mut tiles_query: Query<(&mut Tile, &LayerUpdater)>) {
    tiles_query.par_iter_mut().for_each(|(mut tile, updater)| {
        if let (Some(index), layer) = (updater.index, updater.layer) {
            if let TileTexture::Static(tex) = &mut tile.texture {
                if index >= tex.len() {
                    tex.resize(index + 1, TileLayer::new());
                }
                tex[index] = layer;
            }
        }
        if let Some(color) = updater.color {
            tile.color = color;
        }
    });
}
