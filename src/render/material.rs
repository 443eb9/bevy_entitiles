use std::marker::PhantomData;

use bevy::{
    app::{App, Plugin},
    asset::{Asset, AssetApp},
    core_pipeline::core_2d::Transparent2d,
    ecs::schedule::IntoSystemConfigs,
    reflect::TypePath,
    render::{
        color::Color,
        render_phase::AddRenderCommand,
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, ShaderType,
            SpecializedRenderPipelines,
        },
        ExtractSchedule, Render, RenderApp, RenderSet,
    },
};

use super::{
    binding::TilemapBindGroups,
    buffer::TilemapUniformBuffer,
    chunk::RenderChunkStorage,
    cull,
    draw::{DrawTilemapNonTextured, DrawTilemapTextured},
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

        let render_app = app.sub_app_mut(RenderApp);

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
                    // Splitting into 2 systems because there're
                    // too many parameters for Bevy to handle
                    prepare::prepare_tilemaps_a::<M>,
                    prepare::prepare_tilemaps_b::<M>,
                    prepare::prepare_tiles::<M>,
                    prepare::prepare_unloaded_chunks::<M>,
                    prepare::prepare_despawned_tilemaps::<M>,
                    prepare::prepare_despawned_tiles::<M>,
                    cull::cull_chunks::<M>,
                )
                    .in_set(RenderSet::Prepare),
            )
            .add_systems(Render, queue::queue::<M>.in_set(RenderSet::Queue))
            .init_resource::<RenderChunkStorage<M>>()
            .init_resource::<TilemapUniformBuffer<M>>()
            .init_resource::<TilemapBindGroups<M>>()
            .init_resource::<TilemapInstances<M>>()
            .init_resource::<ExtractedTilemapMaterials<M>>()
            .add_render_command::<Transparent2d, DrawTilemapTextured<M>>()
            .add_render_command::<Transparent2d, DrawTilemapNonTextured<M>>();
    }

    fn finish(&self, app: &mut bevy::prelude::App) {
        let render_app = app.sub_app_mut(RenderApp);

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

    #[allow(unused_variables)]
    fn specialize(descriptor: &mut RenderPipelineDescriptor) {}
}

#[derive(ShaderType)]
pub struct StandardTilemapUniform {
    pub tint: Color,
}

impl From<&StandardTilemapMaterial> for StandardTilemapUniform {
    fn from(value: &StandardTilemapMaterial) -> Self {
        Self { tint: value.tint }
    }
}

#[derive(Default, Asset, AsBindGroup, TypePath, Clone)]
#[uniform(0, StandardTilemapUniform)]
pub struct StandardTilemapMaterial {
    pub tint: Color,
}

impl TilemapMaterial for StandardTilemapMaterial {
    fn vertex_shader() -> ShaderRef {
        super::TILEMAP_SHADER.into()
    }

    fn fragment_shader() -> ShaderRef {
        super::TILEMAP_SHADER.into()
    }
}
