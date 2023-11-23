use bevy::{
    app::Plugin,
    core_pipeline::core_2d::{self, Transparent2d, CORE_2D},
    ecs::{
        entity::Entity,
        reflect::ReflectResource,
        schedule::IntoSystemConfigs,
        system::{Commands, Query, Res, ResMut, Resource},
        world::FromWorld,
    },
    math::Vec2,
    reflect::Reflect,
    render::{
        globals::{GlobalsBuffer, GlobalsUniform},
        render_graph::{RenderGraphApp, ViewNode, ViewNodeRunner},
        render_phase::AddRenderCommand,
        render_resource::{
            BindGroupEntry, BindingResource, DynamicUniformBuffer, Operations, PipelineCache,
            RenderPassColorAttachment, RenderPassDescriptor, ShaderType,
            SpecializedRenderPipelines,
        },
        renderer::{RenderDevice, RenderQueue},
        view::{ViewTarget, ViewUniforms},
        Extract, ExtractSchedule, Render, RenderApp, RenderSet,
    },
};

use crate::{post_processing::PostProcessingTextures, render::draw::DrawTilemapPostProcessing};

use self::pipeline::{MistPipeline, MistPipelineKey};

use super::{
    NoiseData, PostProcessingBindGroups, PostProcessingSettings, PostProcessingView,
    SpecializedPostProcessingPipelines,
};

pub mod pipeline;

pub struct MistPlugin;

impl Plugin for MistPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<FogData>();

        let render_app = app.get_sub_app_mut(RenderApp).unwrap();

        render_app.init_resource::<MistUniformsStorage>();

        render_app.add_render_command::<Transparent2d, DrawTilemapPostProcessing>();
        render_app
            .add_render_graph_node::<ViewNodeRunner<MistNode>>(CORE_2D, MistNode::NAME)
            .add_render_graph_edges(
                CORE_2D,
                &[
                    core_2d::graph::node::MAIN_PASS,
                    MistNode::NAME,
                    core_2d::graph::node::BLOOM,
                ],
            );
    }

    fn finish(&self, _app: &mut bevy::prelude::App) {
        let render_app = _app.sub_app_mut(RenderApp);
        render_app
            .add_systems(ExtractSchedule, extract)
            .add_systems(Render, prepare.in_set(RenderSet::Prepare))
            .add_systems(Render, queue.in_set(RenderSet::Queue));

        render_app
            .init_resource::<SpecializedPostProcessingPipelines>()
            .init_resource::<MistPipeline>()
            .init_resource::<SpecializedRenderPipelines<MistPipeline>>();
    }
}

#[derive(Resource, Clone, Copy, ShaderType)]
pub struct FogData {
    pub min: f32,
    pub max: f32,
    pub octaves: u32,
    pub lacunarity: f32,
    pub gain: f32,
    pub scale: f32,
    pub multiplier: f32,
    pub speed: f32,
}

impl Default for FogData {
    fn default() -> Self {
        Self {
            min: Default::default(),
            max: Default::default(),
            octaves: 5,
            lacunarity: 0.5,
            gain: 1.,
            scale: 0.05,
            multiplier: 0.8,
            speed: 1.,
        }
    }
}

#[derive(Resource, Default)]
pub struct MistUniformsStorage {
    pub fog_buffer: DynamicUniformBuffer<FogData>,
}

pub struct MistNode;

impl MistNode {
    pub const NAME: &str = "mist_post_processing";
}

impl FromWorld for MistNode {
    fn from_world(_world: &mut bevy::prelude::World) -> Self {
        MistNode
    }
}

impl ViewNode for MistNode {
    type ViewQuery = (&'static ViewTarget, &'static PostProcessingView);

    fn run(
        &self,
        _graph: &mut bevy::render::render_graph::RenderGraphContext,
        render_context: &mut bevy::render::renderer::RenderContext,
        (view_target, _): bevy::ecs::query::QueryItem<Self::ViewQuery>,
        world: &bevy::prelude::World,
    ) -> Result<(), bevy::render::render_graph::NodeRunError> {
        let Some(pipeline_id) = world
            .resource::<SpecializedPostProcessingPipelines>()
            .mist_pipeline
        else {
            return Ok(());
        };
        let pipeline = world.resource::<MistPipeline>();
        let bind_groups = world.resource::<PostProcessingBindGroups>();
        let textures = world.resource::<PostProcessingTextures>();

        if bind_groups.screen_height_texture_bind_group.is_none() {
            return Ok(());
        }

        let post_process_write = view_target.post_process_write();
        let screen_color_texture_bind_group = render_context.render_device().create_bind_group(
            "screen_color_texture_bind_group",
            &pipeline.screen_color_texture_layout,
            &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(post_process_write.source),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(
                        textures.screen_color_texture_sampler.as_ref().unwrap(),
                    ),
                },
            ],
        );

        let mut pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("mist_post_processing"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &post_process_write.destination,
                resolve_target: None,
                ops: Operations::default(),
            })],
            depth_stencil_attachment: None,
        });

        let pipeline_cache = world.resource::<PipelineCache>();
        pass.set_render_pipeline(pipeline_cache.get_render_pipeline(pipeline_id).unwrap());
        pass.set_bind_group(
            0,
            bind_groups.uniforms_bind_group.as_ref().unwrap(),
            &[],
        );
        pass.set_bind_group(1, &screen_color_texture_bind_group, &[]);
        pass.set_bind_group(
            2,
            bind_groups
                .screen_height_texture_bind_group
                .as_ref()
                .unwrap(),
            &[],
        );
        pass.set_bind_group(3, bind_groups.fog_uniform_bind_group.as_ref().unwrap(), &[]);
        pass.draw(0..3, 0..1);

        Ok(())
    }
}

pub fn extract(
    mut commands: Commands,
    fog: Extract<Res<FogData>>,
    settings: Extract<Res<PostProcessingSettings>>,
    views_query: Extract<Query<(Entity, &PostProcessingView)>>,
) {
    let mut views = vec![];
    for (entity, _) in views_query.iter() {
        views.push((entity, PostProcessingView));
    }
    commands.insert_or_spawn_batch(views);
    commands.insert_resource(**fog);
    commands.insert_resource(**settings);
}

pub fn prepare(
    render_device: Res<RenderDevice>,
    render_queue: ResMut<RenderQueue>,
    fog: Res<FogData>,
    mut uniform_storage: ResMut<MistUniformsStorage>,
) {
    uniform_storage.fog_buffer.push(*fog);
    uniform_storage
        .fog_buffer
        .write_buffer(&render_device, &render_queue);
}

pub fn queue(
    mut bind_groups: ResMut<PostProcessingBindGroups>,
    uniform_storage: Res<MistUniformsStorage>,
    mut sp_mist_pipeline: ResMut<SpecializedRenderPipelines<MistPipeline>>,
    mist_pipeline: Res<MistPipeline>,
    pipeline_cache: Res<PipelineCache>,
    mut pipelines: ResMut<SpecializedPostProcessingPipelines>,
    render_device: Res<RenderDevice>,
    settings: Res<PostProcessingSettings>,
    globals_uniform: Res<GlobalsBuffer>,
    view_uniform: Res<ViewUniforms>,
) {
    pipelines.mist_pipeline = Some(sp_mist_pipeline.specialize(
        &pipeline_cache,
        &mist_pipeline,
        MistPipelineKey {
            fog: true,
            height_force_display: settings.height_force_display,
        },
    ));

    if let Some(binding) = uniform_storage.fog_buffer.binding() {
        bind_groups.fog_uniform_bind_group = Some(render_device.create_bind_group(
            "fog_uniform_group",
            &mist_pipeline.fog_uniform_layout,
            &[BindGroupEntry {
                binding: 0,
                resource: binding,
            }],
        ));
    }

    if bind_groups.uniforms_bind_group.is_none() {
        if let (Some(globals_bindings), Some(view_bindings)) = (
            globals_uniform.buffer.binding(),
            view_uniform.uniforms.binding(),
        ) {
            bind_groups.uniforms_bind_group = Some(render_device.create_bind_group(
                "globals_uniform_bind_group",
                &mist_pipeline.uniforms_layout,
                &[
                    BindGroupEntry {
                        binding: 0,
                        resource: globals_bindings,
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: view_bindings,
                    },
                ],
            ));
        }
    }
}
