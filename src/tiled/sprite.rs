use bevy::{
    asset::{Asset, Handle},
    reflect::Reflect,
    render::{render_resource::AsBindGroup, texture::Image},
    sprite::Material2d,
};

use super::TILED_SPRITE_SHADER;

#[derive(AsBindGroup, Asset, Debug, Clone, Reflect)]
pub struct TiledSpriteMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub image: Handle<Image>,
}

impl Material2d for TiledSpriteMaterial {
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        TILED_SPRITE_SHADER.into()
    }
}
