use std::marker::PhantomData;

use bevy::{
    app::{App, Plugin},
    asset::{Asset, AssetApp, AssetId},
    color::LinearRgba,
    core_pipeline::core_2d::Transparent2d,
    ecs::{schedule::IntoSystemConfigs, system::SystemParamItem},
    prelude::{Deref, DerefMut},
    reflect::TypePath,
    render::{
        extract_instances::ExtractInstancesPlugin,
        render_asset::{PrepareAssetError, RenderAsset, RenderAssetPlugin},
        render_phase::AddRenderCommand,
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, ShaderType,
            SpecializedRenderPipelines,
        },
        Render, RenderApp, RenderSet,
    },
};

use crate::render::{
    binding::{self, TilemapBindGroups},
    chunk::{self},
    draw::{DrawTilemapNonTextured, DrawTilemapTextured},
    extract::ExtractedTilemap,
    pipeline::EntiTilesPipeline,
    prepare, queue,
};

#[derive(Default)]
pub struct EntiTilesMaterialPlugin<M: TilemapMaterial>(PhantomData<M>);

impl<M: TilemapMaterial> Plugin for EntiTilesMaterialPlugin<M> {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractInstancesPlugin::<ExtractedTilemap>::new(),
            ExtractInstancesPlugin::<AssetId<M>>::new(),
            RenderAssetPlugin::<ExtractedTilemapMaterialWrapper<M>>::default(),
        ))
        .init_asset::<M>();

        let render_app = app.sub_app_mut(RenderApp);

        render_app
            .add_systems(
                Render,
                (
                    prepare::prepare_tiles::<M>,
                    prepare::prepare_unloaded_chunks::<M>,
                    prepare::prepare_despawned_tilemaps::<M>,
                    prepare::prepare_despawned_tiles::<M>,
                    //
                    binding::bind_tilemap_buffers::<M>,
                    binding::bind_materials::<M>,
                    binding::bind_textures::<M>,
                    chunk::prepare_chunks::<M>,
                )
                    .in_set(RenderSet::Prepare),
            )
            .add_systems(
                Render,
                prepare::sort_chunks::<M>.in_set(RenderSet::PrepareResources),
            )
            .add_systems(Render, queue::queue_tilemaps::<M>.in_set(RenderSet::Queue))
            .init_resource::<TilemapBindGroups<M>>()
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

#[derive(Deref, DerefMut)]
pub struct ExtractedTilemapMaterialWrapper<M>(M);

impl<M> RenderAsset for ExtractedTilemapMaterialWrapper<M>
where
    M: TilemapMaterial,
{
    type SourceAsset = M;

    type Param = ();

    #[inline]
    fn prepare_asset(
        source_asset: Self::SourceAsset,
        _param: &mut SystemParamItem<Self::Param>,
    ) -> Result<Self, PrepareAssetError<Self::SourceAsset>> {
        Ok(ExtractedTilemapMaterialWrapper(source_asset))
    }
}

#[derive(ShaderType)]
pub struct StandardTilemapUniform {
    pub tint: LinearRgba,
}

impl From<&StandardTilemapMaterial> for StandardTilemapUniform {
    fn from(value: &StandardTilemapMaterial) -> Self {
        Self { tint: value.tint }
    }
}

#[derive(Default, Asset, AsBindGroup, TypePath, Clone)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
#[uniform(0, StandardTilemapUniform)]
pub struct StandardTilemapMaterial {
    pub tint: LinearRgba,
}

impl TilemapMaterial for StandardTilemapMaterial {
    fn vertex_shader() -> ShaderRef {
        super::TILEMAP_SHADER.into()
    }

    fn fragment_shader() -> ShaderRef {
        super::TILEMAP_SHADER.into()
    }
}
