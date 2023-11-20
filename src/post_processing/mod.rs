use bevy::{
    app::Plugin,
    asset::{load_internal_asset, Handle},
    core_pipeline::core_2d::{
        self,
        graph::node::{BLOOM, MAIN_PASS},
        CORE_2D,
    },
    ecs::component::Component,
    render::{
        render_graph::{RenderGraph, RenderGraphApp},
        render_resource::Shader,
        RenderApp,
    },
};

use self::mist::MistNode;

pub mod mist;
pub mod pipeline;

pub const MIST_SHADER: Handle<Shader> = Handle::weak_from_u128(583415345213345153241);

pub struct EntiTilesPostProcessingPlugin;

impl Plugin for EntiTilesPostProcessingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        load_internal_asset!(app, MIST_SHADER, "mist/mist.wgsl", Shader::from_wgsl);

        // let render_app = app.sub_app_mut(RenderApp);
        // render_app
        //     .add_render_graph_node::<MistNode>(CORE_2D, MistNode::NAME)
        //     .add_render_graph_edges(
        //         CORE_2D,
        //         &[
        //             core_2d::graph::node::MAIN_PASS,
        //             MistNode::NAME,
        //             core_2d::graph::node::BLOOM,
        //         ],
        //     );

        // let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        // render_graph.add_node(MistNode::NAME, MistNode);
        // render_graph.add_node_edge(core_2d::graph::node::MAIN_PASS, MistNode::NAME);
        // render_graph.add_node_edge(MistNode::NAME, core_2d::graph::node::BLOOM);
    }
}

#[derive(Component)]
pub struct PostProcessingView;
