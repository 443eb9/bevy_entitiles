use bevy::{
    ecs::{component::Component, system::Resource, world::FromWorld},
    math::Vec2,
    render::{
        render_graph::Node,
        render_resource::{ComputePipelineDescriptor, PipelineCache, ShaderType},
    },
};

use super::{pipeline::MistPipeline, MIST_SHADER};

#[derive(Resource, Default)]
pub struct TilemapClouds {
    pub clouds: Vec<CloudsData>,
}

pub struct MistNode;

impl MistNode {
    pub const NAME: &str = "mist_post_processing";
}

impl FromWorld for MistNode {
    fn from_world(world: &mut bevy::prelude::World) -> Self {
        MistNode
    }
}

impl Node for MistNode {
    fn update(&mut self, _world: &mut bevy::prelude::World) {
        let mut pipeline = _world.resource_mut::<MistPipeline>();

        // if pipeline.cached_id.is_none() {
        //     let mut pipeline_cache = _world.resource_mut::<PipelineCache>();
        //     pipeline.cached_id = Some(pipeline_cache.queue_compute_pipeline(
        //         ComputePipelineDescriptor {
        //             label: Some("mist_pipeline".into()),
        //             layout: vec![pipeline.mist_layout, pipeline.screen_texture_layout],
        //             push_constant_ranges: vec![],
        //             shader: MIST_SHADER,
        //             shader_defs: vec![],
        //             entry_point: "mist".into(),
        //         },
        //     ));
        // }
    }

    fn run(
        &self,
        graph: &mut bevy::render::render_graph::RenderGraphContext,
        render_context: &mut bevy::render::renderer::RenderContext,
        world: &bevy::prelude::World,
    ) -> Result<(), bevy::render::render_graph::NodeRunError> {
        Ok(())
    }
}

#[derive(Component, ShaderType, Default)]
pub struct CloudsData {
    pub height: f32,
    pub min: Vec2,
    pub max: Vec2,
}
