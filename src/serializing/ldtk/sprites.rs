use bevy::{
    asset::{Asset, Handle},
    math::{IVec2, Vec2},
    reflect::TypePath,
    render::{
        render_resource::{AsBindGroup, ShaderRef, ShaderType},
        texture::Image,
    },
    sprite::Material2d,
};

use super::{json::definitions::TilesetRect, ENTITY_SPRITE_SHADER};

#[derive(ShaderType, Clone, Copy, Debug)]
pub struct AtlasRect {
    pub min: Vec2,
    pub max: Vec2,
}

impl From<TilesetRect> for AtlasRect {
    fn from(value: TilesetRect) -> Self {
        Self {
            min: IVec2::new(value.x_pos, value.y_pos).as_vec2(),
            max: IVec2::new(value.x_pos + value.width, value.y_pos + value.height).as_vec2(),
        }
    }
}

#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub struct LdtkEntityMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub texture: Handle<Image>,
    #[uniform(2)]
    pub atlas_rect: AtlasRect,
}

impl Material2d for LdtkEntityMaterial {
    fn fragment_shader() -> ShaderRef {
        ENTITY_SPRITE_SHADER.into()
    }
}
