use std::marker::PhantomData;

use bevy::{
    app::{App, Plugin},
    asset::{Asset, AssetApp, Assets, Handle},
    core_pipeline::core_2d::Transparent2d,
    ecs::{
        component::Component,
        entity::Entity,
        query::With,
        schedule::IntoSystemConfigs,
        system::{Commands, Query, ResMut, Resource},
    },
    reflect::TypePath,
    render::{
        render_phase::AddRenderCommand,
        render_resource::{AsBindGroup, ShaderRef, SpecializedRenderPipelines},
        ExtractSchedule, Render, RenderApp, RenderSet,
    },
};

use super::{
    binding::TilemapBindGroups,
    buffer::TilemapUniformBuffer,
    chunk::RenderChunkStorage,
    cull,
    draw::DrawTilemap,
    extract,
    pipeline::EntiTilesPipeline,
    prepare, queue,
    resources::{ExtractedTilemapMaterials, TilemapInstances},
};

#[derive(Default)]
pub struct EntiTilesMaterialPlugin<M: TilemapMaterial>(PhantomData<M>);

impl<M: TilemapMaterial> Plugin for EntiTilesMaterialPlugin<M> {
    fn build(&self, app: &mut App) {
        app.init_asset::<M>();

        let render_app = app.get_sub_app_mut(RenderApp).unwrap();

        render_app
            .add_systems(
                ExtractSchedule,
                (
                    extract::extract_changed_tilemaps::<M>,
                    extract::extract_materials::<M>,
                ),
            )
            .add_systems(
                Render,
                (
                    prepare::prepare_tilemaps::<M>,
                    prepare::prepare_tiles::<M>,
                    prepare::prepare_unloaded_chunks::<M>,
                    prepare::prepare_despawned_tilemaps::<M>,
                    prepare::prepare_despawned_tiles::<M>,
                    cull::cull_chunks::<M>,
                )
                    .in_set(RenderSet::Prepare),
            )
            .add_systems(Render, queue::queue::<M>.in_set(RenderSet::Queue));

        render_app
            .init_resource::<RenderChunkStorage<M>>()
            .init_resource::<TilemapUniformBuffer<M>>()
            .init_resource::<TilemapBindGroups<M>>()
            .init_resource::<TilemapInstances<M>>()
            .init_resource::<ExtractedTilemapMaterials<M>>();

        render_app.add_render_command::<Transparent2d, DrawTilemap<M>>();
    }

    fn finish(&self, app: &mut bevy::prelude::App) {
        let render_app = app.get_sub_app_mut(RenderApp).unwrap();

        render_app
            .init_resource::<EntiTilesPipeline<M>>()
            .init_resource::<SpecializedRenderPipelines<EntiTilesPipeline<M>>>();
    }
}

pub trait TilemapMaterial: Default + Asset + AsBindGroup + TypePath + Clone {
    fn vertex_shader() -> ShaderRef {
        super::TILEMAP_SHADER.into()
    }

    fn fragment_shader() -> ShaderRef {
        super::TILEMAP_SHADER.into()
    }
}

#[derive(Component, Default, Debug, Clone)]
pub struct WaitForStandardMaterialReplacement;

#[derive(Resource, Default)]
pub struct StandardTilemapMaterialSingleton(pub Option<Handle<StandardTilemapMaterial>>);

#[derive(Default, Asset, AsBindGroup, TypePath, Clone)]
pub struct StandardTilemapMaterial {}

impl TilemapMaterial for StandardTilemapMaterial {
    fn vertex_shader() -> ShaderRef {
        super::TILEMAP_SHADER.into()
    }

    fn fragment_shader() -> ShaderRef {
        super::TILEMAP_SHADER.into()
    }
}

pub fn standard_material_register(
    mut commands: Commands,
    mut tilemaps_query: Query<
        (Entity, &mut Handle<StandardTilemapMaterial>),
        With<WaitForStandardMaterialReplacement>,
    >,
    mut materials: ResMut<Assets<StandardTilemapMaterial>>,
    mut material_singleton: ResMut<StandardTilemapMaterialSingleton>,
) {
    if material_singleton.0.is_none() {
        let material = materials.add(StandardTilemapMaterial::default());
        material_singleton.0 = Some(material);
    }

    for (entity, mut material) in tilemaps_query.iter_mut() {
        *material = material_singleton.0.clone().unwrap();
        commands
            .entity(entity)
            .remove::<WaitForStandardMaterialReplacement>();
    }
}
