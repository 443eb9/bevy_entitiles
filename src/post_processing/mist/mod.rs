use bevy::{
    ecs::{
        system::{Commands, Res, Resource},
        world::FromWorld,
    },
    math::Vec3,
    render::{
        render_graph::{Node, ViewNode},
        render_resource::{BindGroup, ShaderType},
        Extract,
    },
};

use super::PostProcessingView;

pub mod pipeline;

#[derive(Resource, Default, Clone, Copy, ShaderType)]
pub struct FogData {
    pub min: f32,
    pub max: f32,
}

#[derive(Resource)]
pub struct MistBindGroups {
    /// The config
    pub mist_group: BindGroup,
}

pub fn extract(mut commands: Commands, fog: Extract<Res<FogData>>) {
    commands.insert_resource(**fog);
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

impl ViewNode for MistNode {
    type ViewQuery = &'static PostProcessingView;

    fn run(
        &self,
        graph: &mut bevy::render::render_graph::RenderGraphContext,
        render_context: &mut bevy::render::renderer::RenderContext,
        view_query: bevy::ecs::query::QueryItem<Self::ViewQuery>,
        world: &bevy::prelude::World,
    ) -> Result<(), bevy::render::render_graph::NodeRunError> {
        todo!()
    }
}
