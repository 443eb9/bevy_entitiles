use bevy::{asset::Handle, ecs::component::Component, render::texture::Image};

use crate::tilemap::{map::Tilemap, tile::TilemapTexture};

/// This tilemap will be used when rendering volumetric clouds/fog or SSAO.
#[derive(Component)]
pub struct HeightTilemap {
    pub(crate) height_map: TilemapTexture,
}

impl HeightTilemap {
    pub fn new(height_map: Handle<Image>, tilemap: &Tilemap) -> Self {
        if let Some(tex) = &tilemap.texture {
            Self {
                height_map: TilemapTexture {
                    texture: height_map,
                    desc: tex.desc.clone(),
                },
            }
        } else {
            panic!("Tilemap texture must be set before creating a height tilemap.")
        }
    }
}
