use bevy::{asset::Handle, ecs::component::Component, render::texture::Image};

use crate::tilemap::{map::Tilemap, tile::TilemapTexture};

/// This tilemap will be used when rendering volumetric fog and SSAO.
#[derive(Component)]
pub struct HeightTilemap {
    pub(crate) height_texture: TilemapTexture,
}

impl HeightTilemap {
    pub fn new(height_texture: Handle<Image>, tilemap: &Tilemap) -> Self {
        if let Some(tex) = &tilemap.texture {
            Self {
                height_texture: TilemapTexture {
                    texture: height_texture,
                    desc: tex.desc.clone(),
                },
            }
        } else {
            panic!("Tilemap texture must be set before creating a height tilemap.")
        }
    }
}
