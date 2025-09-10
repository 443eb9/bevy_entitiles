use bevy::{
    asset::{Asset, Handle},
    math::Vec4,
    prelude::Image,
    reflect::Reflect,
    render::render_resource::{AsBindGroup, ShaderType},
    sprite::Material2d,
};

use crate::tiled::TILED_SPRITE_SHADER;

#[derive(ShaderType, Debug, Clone, Reflect)]
pub struct SpriteUniform {
    /// min max
    pub atlas: Vec4,
    pub tint: Vec4,
}

#[derive(AsBindGroup, Asset, Debug, Clone, Reflect)]
pub struct TiledSpriteMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub image: Handle<Image>,
    #[uniform(2)]
    pub data: SpriteUniform,
}

impl Material2d for TiledSpriteMaterial {
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        TILED_SPRITE_SHADER.into()
    }
}
