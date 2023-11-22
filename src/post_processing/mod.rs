use bevy::{
    app::Plugin,
    asset::{load_internal_asset, Handle},
    ecs::{
        component::Component,
        schedule::IntoSystemConfigs,
        system::Resource,
    },
    render::{
        render_resource::{
            BindGroup, CachedRenderPipelineId, FilterMode, Sampler,
            Shader,
        },
        texture::{GpuImage, Image},
        ExtractSchedule, Render, RenderApp, RenderSet,
    },
    utils::HashMap,
};

use crate::post_processing::{
        mist::MistPlugin,
        stages::{
            extract_height_maps, prepare_post_processing, prepare_post_processing_textures,
        },
    };

pub mod mist;
pub mod stages;

pub const MIST_SHADER: Handle<Shader> = Handle::weak_from_u128(583415345213345153241);

pub struct EntiTilesPostProcessingPlugin;

impl Plugin for EntiTilesPostProcessingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        load_internal_asset!(app, MIST_SHADER, "mist/mist.wgsl", Shader::from_wgsl);
        app.add_plugins(MistPlugin);

        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .add_systems(ExtractSchedule, extract_height_maps)
            .add_systems(
                Render,
                (
                    prepare_post_processing,
                    prepare_post_processing_textures,
                )
                    .in_set(RenderSet::Prepare),
            );

        render_app
            .init_resource::<PostProcessingTextures>()
            .init_resource::<PostProcessingSettings>()
            .init_resource::<PostProcessingBindGroups>();
    }
}

#[derive(Resource, Default)]
pub struct PostProcessingSettings {
    pub filter_mode: FilterMode,
}

#[derive(Component)]
pub struct PostProcessingView;

#[derive(Resource, Default)]
pub struct PostProcessingBindGroups {
    /// All the height textures
    pub height_texture_bind_groups: HashMap<Handle<Image>, BindGroup>,
    /// A texture contains the height data
    pub screen_height_texture_bind_group: Option<BindGroup>,
    /// The rendered screen texture
    pub screen_color_texture_bind_group: Option<BindGroup>,
    pub fog_uniform_bind_group: Option<BindGroup>,
}

#[derive(Resource, Default)]
pub struct PostProcessingTextures {
    pub screen_color_texture_sampler: Option<Sampler>,
    pub screen_height_texture: Handle<Image>,
    pub screen_height_gpu_image: Option<GpuImage>,
}

#[derive(Resource, Default)]
pub struct SpecializedPostProcessingPipelines {
    pub mist_pipeline: Option<CachedRenderPipelineId>,
}
