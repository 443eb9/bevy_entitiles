use bevy::{
    asset::{Asset, Handle},
    math::{Vec2, Vec4},
    reflect::Reflect,
    render::{
        render_resource::{AsBindGroup, ShaderType},
        texture::Image,
    },
    sprite::Material2d,
};

use crate::tiled::TILED_SPRITE_SHADER;

#[derive(ShaderType, Debug, Clone, Reflect)]
pub struct SpriteUniform {
    /// min max
    pub atlas: [Vec2; 2],
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
