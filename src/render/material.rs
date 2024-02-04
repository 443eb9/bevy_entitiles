use std::marker::PhantomData;

use bevy::{
    app::{App, Plugin},
    asset::{Asset, Handle},
    ecs::{
        entity::Entity,
        query::Changed,
        system::{Query, ResMut},
    },
    reflect::Reflect,
    render::{
        render_resource::{AsBindGroup, ShaderRef},
        Extract,
    },
};

use crate::tilemap::map::{TilemapAnimations, TilemapTexture};

use super::resources::TilemapMaterialInstances;

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
pub trait TilemapMaterial: Asset + AsBindGroup {
    fn vertex_shader_ovrd() -> Option<ShaderRef> {
        None
    }

    fn fragment_shader_ovrd() -> Option<ShaderRef> {
        None
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

impl TilemapMaterial for TextureOnlyTilemapMaterial {
    fn texture(&self) -> Option<&TilemapTexture> {
        Some(&self.texture)
    }
}

#[derive(Asset, AsBindGroup, Clone, Reflect)]
pub struct PureColorTilemapMaterial {}

impl TilemapMaterial for PureColorTilemapMaterial {}

pub fn extract<M: TilemapMaterial>(
    tilemaps_query: Extract<Query<(Entity, &Handle<M>), Changed<Handle<M>>>>,
    mut material_instances: ResMut<TilemapMaterialInstances<M>>,
) {
    tilemaps_query.for_each(|(entity, handle)| {
        material_instances.0.insert(entity, handle.id());
    });
}
