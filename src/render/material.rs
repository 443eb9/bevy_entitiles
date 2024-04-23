use std::marker::PhantomData;

use bevy::{
    app::{App, Plugin},
    asset::{Asset, AssetApp, AssetId},
    core_pipeline::core_2d::Transparent2d,
    ecs::{schedule::IntoSystemConfigs, system::Resource},
    reflect::TypePath,
    render::{
        color::Color,
        render_asset::prepare_assets,
        render_phase::AddRenderCommand,
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedRenderPipelines,
        },
        texture::Image,
        ExtractSchedule, Render, RenderApp, RenderSet,
    },
    sprite::{Material2d, PreparedMaterial2d},
    utils::HashMap,
};

use crate::tilemap::map::TilemapTexture;

use super::{
    binding::TilemapBindGroups,
    draw::{
        DrawMaterialTilemap, DrawMaterialTilemapWithoutTexture, DrawTilemap,
        DrawTilemapWithoutTexture,
    },
    extract,
    pipeline::EntiTilesPipeline,
    prepare, queue,
};

#[derive(Default)]
pub struct EntiTilesAdditionalMaterialPlugin<M: TilemapMaterial>(PhantomData<M>);

impl<M: TilemapMaterial> Plugin for EntiTilesAdditionalMaterialPlugin<M> {
    fn build(&self, app: &mut App) {
        app.init_asset::<M>();

        let render_app = app.sub_app_mut(RenderApp);

        render_app
            .init_resource::<TilemapMaterialInstances<M>>()
            .init_resource::<TilemapBindGroups<M>>()
            .add_systems(ExtractSchedule, extract::extract_tilemap_materials::<M>)
            .add_systems(
                Render,
                prepare::prepare_tilemap_materials::<M>
                    .in_set(RenderSet::Prepare)
                    .after(prepare_assets::<Image>),
            )
            .add_systems(Render, queue::queue::<M>.in_set(RenderSet::Queue))
            .add_render_command::<Transparent2d, DrawMaterialTilemap<M>>()
            .add_render_command::<Transparent2d, DrawMaterialTilemapWithoutTexture<M>>();
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);

        render_app
            .init_resource::<EntiTilesPipeline<M>>()
            .init_resource::<SpecializedRenderPipelines<EntiTilesPipeline<M>>>();
    }
}

pub struct StandardTilemapMaterialPlugin;

impl Plugin for StandardTilemapMaterialPlugin {
    fn build(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);

        render_app
            .init_resource::<TilemapMaterialInstances<StandardTilemapMaterial>>()
            .init_resource::<TilemapBindGroups<StandardTilemapMaterial>>()
            .add_systems(
                ExtractSchedule,
                extract::extract_tilemap_materials::<StandardTilemapMaterial>,
            )
            .add_systems(
                Render,
                prepare::prepare_std_materials.in_set(RenderSet::Prepare),
            )
            .add_systems(
                Render,
                queue::queue::<StandardTilemapMaterial>.in_set(RenderSet::Queue),
            )
            .add_render_command::<Transparent2d, DrawTilemap>()
            .add_render_command::<Transparent2d, DrawTilemapWithoutTexture>();
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);

        render_app
            .init_resource::<EntiTilesPipeline<StandardTilemapMaterial>>()
            .init_resource::<SpecializedRenderPipelines<EntiTilesPipeline<StandardTilemapMaterial>>>();
    }
}

pub trait TilemapMaterial: Asset + AsBindGroup + Clone {
    fn vertex_shader() -> ShaderRef {
        ShaderRef::Default
    }

    fn fragment_shader() -> ShaderRef {
        ShaderRef::Default
    }

    fn specialize(_desc: &mut RenderPipelineDescriptor) {}
}

#[derive(Default, Asset, TypePath, Clone)]
pub struct StandardTilemapMaterial {
    pub tint: Color,
    pub texture: Option<TilemapTexture>,
}

impl AsBindGroup for StandardTilemapMaterial {
    type Data = ();

    fn bind_group_layout_entries(
        _render_device: &bevy::render::renderer::RenderDevice,
    ) -> Vec<bevy::render::render_resource::BindGroupLayoutEntry>
    where
        Self: Sized,
    {
        vec![]
    }

    fn unprepared_bind_group(
        &self,
        _layout: &bevy::render::render_resource::BindGroupLayout,
        _render_device: &bevy::render::renderer::RenderDevice,
        _images: &bevy::render::render_asset::RenderAssets<Image>,
        _fallback_image: &bevy::render::texture::FallbackImage,
    ) -> Result<
        bevy::render::render_resource::UnpreparedBindGroup<Self::Data>,
        bevy::render::render_resource::AsBindGroupError,
    > {
        Err(bevy::render::render_resource::AsBindGroupError::RetryNextUpdate)
    }
}

impl TilemapMaterial for StandardTilemapMaterial {}

#[derive(Resource)]
pub struct TilemapMaterialInstances<M: TilemapMaterial> {
    instances: HashMap<AssetId<M>, M>,
}

impl<M: TilemapMaterial> Default for TilemapMaterialInstances<M> {
    fn default() -> Self {
        Self {
            instances: Default::default(),
        }
    }
}

impl<M: TilemapMaterial> TilemapMaterialInstances<M> {
    #[inline]
    pub fn get(&self, handle: &AssetId<M>) -> Option<&M> {
        self.instances.get(handle)
    }

    #[inline]
    pub fn remove(&mut self, handle: &AssetId<M>) {
        self.instances.remove(handle);
    }

    #[inline]
    pub fn insert(&mut self, handle: AssetId<M>, material: M) {
        self.instances.insert(handle, material);
    }
}

#[derive(Resource, Default)]
pub struct ExtractedTilemapMaterials<M: TilemapMaterial> {
    pub extracted: Vec<(AssetId<M>, M)>,
    pub removed: Vec<AssetId<M>>,
}

pub struct PrepareNextFrameTilemapMaterials<M: TilemapMaterial> {
    pub assets: Vec<(AssetId<M>, M)>,
}

impl<M: TilemapMaterial> Default for PrepareNextFrameTilemapMaterials<M> {
    fn default() -> Self {
        Self {
            assets: Default::default(),
        }
    }
}

#[derive(Resource, Default)]
pub struct RenderMaterials2d<T: Material2d>(pub HashMap<AssetId<T>, PreparedMaterial2d<T>>);
