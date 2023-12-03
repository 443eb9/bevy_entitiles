use bevy::{
    asset::Handle,
    ecs::{
        component::Component,
        entity::Entity,
        system::{
            lifetimeless::{Read, SRes},
            Commands, Query, Res, ResMut,
        },
    },
    math::Vec2,
    render::{
        render_phase::{RenderCommand, RenderCommandResult},
        render_resource::FilterMode,
        renderer::{RenderDevice, RenderQueue},
        texture::Image,
        Extract,
    },
    transform::commands,
    ui::TransparentUi,
};

use crate::{
    render::{
        texture::TilemapTextureDescriptor,
        uniform::{DynamicUniformComponent, UniformsStorage},
    },
    tilemap::{tile::TilemapTexture, ui::UiTilemap},
};

use super::{
    resources::UiBindGroups,
    uniform::{UiTilemapUniform, UiTilemapUniformsStorage},
};

#[derive(Component)]
pub struct ExtractedUiTilemap {
    pub anchor: Vec2,
    pub texture: TilemapTexture,
}

pub fn extract(mut commands: Commands, tilemaps_query: Extract<Query<(Entity, &UiTilemap)>>) {
    let mut tilemaps = vec![];
    for (entity, tilemap) in tilemaps_query.iter() {
        tilemaps.push((
            entity,
            ExtractedUiTilemap {
                anchor: tilemap.anchor,
                texture: tilemap.texture.clone(),
            },
        ))
    }
    commands.insert_or_spawn_batch(tilemaps);
}

pub fn prepare(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut uniforms: ResMut<UiTilemapUniformsStorage>,
    tilemaps_query: Query<(Entity, &ExtractedUiTilemap)>,
) {
    uniforms.clear();
    for (entity, tilemap) in tilemaps_query.iter() {
        commands.entity(entity).insert(uniforms.insert(tilemap));
    }
    uniforms.write(&render_device, &render_queue);
}

pub fn queue() {}

pub struct SetUiTilemapTextureBindGroup<const I: usize>;
impl<const I: usize> RenderCommand<TransparentUi> for SetUiTilemapTextureBindGroup<I> {
    type Param = SRes<UiBindGroups>;

    type ViewWorldQuery = ();

    type ItemWorldQuery = Read<ExtractedUiTilemap>;

    fn render<'w>(
        _item: &TransparentUi,
        _view: bevy::ecs::query::ROQueryItem<'w, Self::ViewWorldQuery>,
        tilemap: bevy::ecs::query::ROQueryItem<'w, Self::ItemWorldQuery>,
        bind_groups: bevy::ecs::system::SystemParamItem<'w, '_, Self::Param>,
        pass: &mut bevy::render::render_phase::TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        pass.set_bind_group(
            I,
            bind_groups
                .into_inner()
                .colored_texture
                .get(tilemap.texture.handle())
                .unwrap(),
            &[],
        );

        RenderCommandResult::Success
    }
}

pub struct SetUiTilemapUniformBindGroup<const I: usize>;
impl<const I: usize> RenderCommand<TransparentUi> for SetUiTilemapUniformBindGroup<I> {
    type Param = SRes<UiBindGroups>;

    type ViewWorldQuery = ();

    type ItemWorldQuery = Read<DynamicUniformComponent<UiTilemapUniform>>;

    fn render<'w>(
        item: &TransparentUi,
        _view: bevy::ecs::query::ROQueryItem<'w, Self::ViewWorldQuery>,
        uniform: bevy::ecs::query::ROQueryItem<'w, Self::ItemWorldQuery>,
        bind_groups: bevy::ecs::system::SystemParamItem<'w, '_, Self::Param>,
        pass: &mut bevy::render::render_phase::TrackedRenderPass<'w>,
    ) -> bevy::render::render_phase::RenderCommandResult {
        pass.set_bind_group(
            I,
            bind_groups
                .into_inner()
                .ui_tilemap_uniforms
                .get(&item.entity)
                .unwrap(),
            &[uniform.index],
        );

        RenderCommandResult::Success
    }
}

pub struct DrawUiTileMesh<const I: usize>;
impl<const I: usize> RenderCommand<TransparentUi> for DrawUiTileMesh<I> {
    type Param = ();

    type ViewWorldQuery = ();

    type ItemWorldQuery = ();

    fn render<'w>(
        item: &TransparentUi,
        view: bevy::ecs::query::ROQueryItem<'w, Self::ViewWorldQuery>,
        entity: bevy::ecs::query::ROQueryItem<'w, Self::ItemWorldQuery>,
        param: bevy::ecs::system::SystemParamItem<'w, '_, Self::Param>,
        pass: &mut bevy::render::render_phase::TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        
    }
}
