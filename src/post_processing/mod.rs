use bevy::{
    app::Plugin,
    asset::{load_internal_asset, Handle},
    core_pipeline::core_2d::{self, CORE_2D},
    ecs::{
        component::Component,
        system::{Res, ResMut},
    },
    render::{
        render_graph::{RenderGraphApp, ViewNodeRunner},
        render_resource::{BindGroup, BindGroupEntry, Shader},
        renderer::RenderDevice,
        RenderApp,
    },
};

use crate::render::pipeline::EntiTilesPipeline;

use self::mist::MistNode;

pub mod mist;

pub const MIST_SHADER: Handle<Shader> = Handle::weak_from_u128(583415345213345153241);

pub struct EntiTilesPostProcessingPlugin;

impl Plugin for EntiTilesPostProcessingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        load_internal_asset!(app, MIST_SHADER, "mist/mist.wgsl", Shader::from_wgsl);

        let render_app = app.sub_app_mut(RenderApp);
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
}

#[derive(Component)]
pub struct PostProcessingView;

pub struct PostProcessBindGroups {
    /// A texture contains the height data
    pub height_texture_bind_group: BindGroup,
    /// The rendered screen texture
    pub screen_texture_bind_group: BindGroup,
}

fn queue(mut render_device: ResMut<RenderDevice>, pipeline: Res<EntiTilesPipeline>) {}
