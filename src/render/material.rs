use std::marker::PhantomData;

use bevy::{
    app::{App, Plugin},
    asset::Asset,
    reflect::Reflect,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::Material2d,
};

use crate::tilemap::map::{TilemapAnimations, TilemapTexture};

pub struct TilemapMaterialPlugin<M: TilemapMaterial>(PhantomData<M>);

impl<M: TilemapMaterial> Plugin for TilemapMaterialPlugin<M> {
    fn build(&self, _app: &mut App) {}
}

impl<M: TilemapMaterial> Default for TilemapMaterialPlugin<M> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

/// Trait for tilemap materials. Implement this for your custom tilemap materials
/// and add them to your tilemap.
pub trait TilemapMaterial: Material2d {
    fn vertex_shader() -> ShaderRef {
        super::TILEMAP_SHADER.into()
    }

    fn fragment_shader() -> ShaderRef {
        super::TILEMAP_SHADER.into()
    }

    fn texture(&self) -> Option<&TilemapTexture> {
        None
    }

    fn animations(&self) -> Option<&TilemapAnimations> {
        None
    }
}

#[derive(Asset, AsBindGroup, Clone, Reflect)]
pub struct StandardTilemapMaterial {
    pub texture: TilemapTexture,
    pub animations: TilemapAnimations,
}

impl Material2d for StandardTilemapMaterial {}

impl TilemapMaterial for StandardTilemapMaterial {
    fn texture(&self) -> Option<&TilemapTexture> {
        Some(&self.texture)
    }

    fn animations(&self) -> Option<&TilemapAnimations> {
        Some(&self.animations)
    }
}

#[derive(Asset, AsBindGroup, Clone, Reflect)]
pub struct TextureOnlyTilemapMaterial {
    pub texture: TilemapTexture,
}

impl Material2d for TextureOnlyTilemapMaterial {}

impl TilemapMaterial for TextureOnlyTilemapMaterial {
    fn texture(&self) -> Option<&TilemapTexture> {
        Some(&self.texture)
    }
}

#[derive(Asset, AsBindGroup, Clone, Reflect)]
pub struct PureColorTilemapMaterial {}

impl Material2d for PureColorTilemapMaterial {}

impl TilemapMaterial for PureColorTilemapMaterial {}

pub fn extract() {}
