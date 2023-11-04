use bevy::{
    asset::load_internal_asset,
    core_pipeline::core_2d::Transparent2d,
    prelude::{Entity, Handle, HandleUntyped, Image, IntoSystemConfigs, Plugin, Resource, Shader},
    reflect::TypeUuid,
    render::{
        render_phase::AddRenderCommand,
        render_resource::{BindGroup, SpecializedRenderPipelines},
        ExtractSchedule, Render, RenderApp, RenderSet,
    },
    utils::HashMap,
};

use crate::render::{
    chunk::RenderChunkStorage,
    draw::{DrawTilemap, DrawTilemapPureColor},
    pipeline::EntiTilesPipeline,
    texture::TilemapTextureArrayStorage,
    uniform::TilemapUniformsStorage,
};

pub mod chunk;
pub mod culling;
pub mod draw;
pub mod extract;
pub mod pipeline;
pub mod prepare;
pub mod queue;
pub mod texture;
pub mod uniform;

const SQUARE: HandleUntyped = HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 4189641863548);
const ISO_DIAMOND: HandleUntyped = HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 65416516351);
const COMMON: HandleUntyped = HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 548635415641535);
const TILEMAP_SHADER: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 151631653416315);

pub struct EntiTilesRendererPlugin;

impl Plugin for EntiTilesRendererPlugin {
    fn build(&self, _app: &mut bevy::prelude::App) {}

    fn finish(&self, _app: &mut bevy::prelude::App) {
        load_internal_asset!(_app, SQUARE, "shaders/square.wgsl", Shader::from_wgsl);
        load_internal_asset!(
            _app,
            ISO_DIAMOND,
            "shaders/iso_diamond.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(_app, COMMON, "shaders/common.wgsl", Shader::from_wgsl);

        load_internal_asset!(
            _app,
            TILEMAP_SHADER,
            "shaders/tilemap.wgsl",
            Shader::from_wgsl
        );

        let render_app = _app.get_sub_app_mut(RenderApp).unwrap();

        render_app
            .add_systems(
                ExtractSchedule,
                (extract::extract_tilemaps, extract::extract_camera),
            )
            .add_systems(
                Render,
                (
                    prepare::prepare,
                    culling::cull_tilemaps,
                    culling::cull_chunks,
                )
                    .in_set(RenderSet::Prepare),
            )
            .add_systems(Render, queue::queue.in_set(RenderSet::Queue));

        render_app
            .init_resource::<RenderChunkStorage>()
            .init_resource::<TilemapTextureArrayStorage>()
            .init_resource::<TilemapUniformsStorage>()
            .init_resource::<EntiTilesPipeline>()
            .init_resource::<BindGroups>()
            .init_resource::<SpecializedRenderPipelines<EntiTilesPipeline>>();

        render_app.add_render_command::<Transparent2d, DrawTilemap>();
        render_app.add_render_command::<Transparent2d, DrawTilemapPureColor>();
    }
}

#[derive(Resource, Default)]
pub struct BindGroups {
    pub tilemap_uniform_bind_group: HashMap<Entity, BindGroup>,
    pub tilemap_texture_arrays: HashMap<Handle<Image>, BindGroup>,
}
