use bevy::{
    asset::load_internal_asset,
    core_pipeline::core_2d::Transparent2d,
    prelude::{Handle, IntoSystemConfigs, Plugin, Shader},
    render::{
        mesh::MeshVertexAttribute,
        render_phase::AddRenderCommand,
        render_resource::{SpecializedRenderPipelines, VertexFormat},
        ExtractSchedule, Render, RenderApp, RenderSet,
    },
};

use crate::render::{
    binding::{TilemapBindGroupLayouts, TilemapBindGroups},
    buffer::{TilemapStorageBuffers, TilemapUniformBuffers},
    chunk::RenderChunkStorage,
    draw::{DrawTilemap, DrawTilemapPureColor},
    pipeline::EntiTilesPipeline,
    texture::TilemapTexturesStorage,
};

pub mod binding;
pub mod buffer;
pub mod chunk;
pub mod culling;
pub mod draw;
pub mod extract;
pub mod pipeline;
pub mod prepare;
pub mod queue;
pub mod texture;

const SQUARE: Handle<Shader> = Handle::weak_from_u128(54311635145631);
const ISOMETRIC: Handle<Shader> = Handle::weak_from_u128(45522415151365135);
const HEXAGONAL: Handle<Shader> = Handle::weak_from_u128(341658413214563135);
const COMMON: Handle<Shader> = Handle::weak_from_u128(1321023135616351);
const TILEMAP_SHADER: Handle<Shader> = Handle::weak_from_u128(89646584153215);

pub const TILEMAP_MESH_ATTR_INDEX: MeshVertexAttribute =
    MeshVertexAttribute::new("GridIndex", 14513156146, VertexFormat::Sint32x3);
pub const TILEMAP_MESH_ATTR_COLOR: MeshVertexAttribute =
    MeshVertexAttribute::new("Color", 85415341854, VertexFormat::Float32x4);
pub const TILEMAP_MESH_ATTR_TEX_INDICES: MeshVertexAttribute =
    MeshVertexAttribute::new("TextureIndex", 186541653135, VertexFormat::Sint32x4);
pub const TILEMAP_MESH_ATTR_FLIP: MeshVertexAttribute =
    MeshVertexAttribute::new("Flip", 7365156123161, VertexFormat::Uint32x4);

pub struct EntiTilesRendererPlugin;

impl Plugin for EntiTilesRendererPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        load_internal_asset!(app, SQUARE, "shaders/square.wgsl", Shader::from_wgsl);
        load_internal_asset!(app, ISOMETRIC, "shaders/isometric.wgsl", Shader::from_wgsl);
        load_internal_asset!(app, HEXAGONAL, "shaders/hexagonal.wgsl", Shader::from_wgsl);
        load_internal_asset!(app, COMMON, "shaders/common.wgsl", Shader::from_wgsl);

        // TODO enable this after the panic issue is solved
        // app.add_systems(Update, culling::cull_tilemaps);

        load_internal_asset!(
            app,
            TILEMAP_SHADER,
            "shaders/tilemap.wgsl",
            Shader::from_wgsl
        );

        let render_app = app.get_sub_app_mut(RenderApp).unwrap();

        render_app
            .add_systems(
                ExtractSchedule,
                (
                    extract::extract_tilemaps,
                    extract::extract_tiles,
                    extract::extract_view,
                    extract::extract_unloaded_chunks,
                ),
            )
            .add_systems(
                Render,
                (
                    prepare::prepare_tilemaps,
                    prepare::prepare_tiles,
                    prepare::prepare_unloaded_chunks,
                    culling::cull_chunks,
                )
                    .in_set(RenderSet::Prepare),
            )
            .add_systems(Render, queue::queue.in_set(RenderSet::Queue));

        render_app
            .init_resource::<RenderChunkStorage>()
            .init_resource::<TilemapTexturesStorage>()
            .init_resource::<TilemapUniformBuffers>()
            .init_resource::<TilemapStorageBuffers>()
            .init_resource::<TilemapBindGroups>();

        render_app
            .add_render_command::<Transparent2d, DrawTilemap>()
            .add_render_command::<Transparent2d, DrawTilemapPureColor>();
    }

    fn finish(&self, app: &mut bevy::prelude::App) {
        let render_app = app.get_sub_app_mut(RenderApp).unwrap();

        render_app
            .init_resource::<TilemapBindGroupLayouts>()
            .init_resource::<EntiTilesPipeline>()
            .init_resource::<SpecializedRenderPipelines<EntiTilesPipeline>>();
    }
}
