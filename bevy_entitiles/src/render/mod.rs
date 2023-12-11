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
    chunk::RenderChunkStorage,
    draw::{DrawTilemap, DrawTilemapPureColor},
    pipeline::EntiTilesPipeline,
    texture::TilemapTexturesStorage,
    uniform::TilemapUniformsStorage, resources::{TilemapBindGroups, TilemapBindGroupLayouts},
};

pub mod chunk;
pub mod culling;
pub mod draw;
pub mod extract;
pub mod pipeline;
pub mod prepare;
pub mod queue;
pub mod resources;
pub mod texture;
pub mod uniform;

const SQUARE: Handle<Shader> = Handle::weak_from_u128(54311635145631);
const ISO_DIAMOND: Handle<Shader> = Handle::weak_from_u128(45522415151365135);
const COMMON: Handle<Shader> = Handle::weak_from_u128(1321023135616351);
const TILEMAP_SHADER: Handle<Shader> = Handle::weak_from_u128(89646584153215);

pub const TILEMAP_MESH_ATTR_INDEX: MeshVertexAttribute =
    MeshVertexAttribute::new("GridIndex", 14513156146, VertexFormat::Float32x3);
pub const TILEMAP_MESH_ATTR_COLOR: MeshVertexAttribute =
    MeshVertexAttribute::new("Color", 85415341854, VertexFormat::Float32x4);
pub const TILEMAP_MESH_ATTR_ATLAS_INDICES: MeshVertexAttribute =
    MeshVertexAttribute::new("AtlasIndex", 186541653135, VertexFormat::Sint32x4);
pub const TILEMAP_MESH_ATTR_RD_SIZE: MeshVertexAttribute =
    MeshVertexAttribute::new("RenderSize", 54164856348564, VertexFormat::Float32x2);

pub struct EntiTilesRendererPlugin;

impl Plugin for EntiTilesRendererPlugin {
    fn build(&self, _app: &mut bevy::prelude::App) {}

    fn finish(&self, app: &mut bevy::prelude::App) {
        load_internal_asset!(app, SQUARE, "shaders/square.wgsl", Shader::from_wgsl);
        load_internal_asset!(
            app,
            ISO_DIAMOND,
            "shaders/iso_diamond.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(app, COMMON, "shaders/common.wgsl", Shader::from_wgsl);

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
                ),
            )
            .add_systems(
                Render,
                (prepare::prepare, culling::cull).in_set(RenderSet::Prepare),
            )
            .add_systems(Render, queue::queue.in_set(RenderSet::Queue));

        render_app
            .init_resource::<RenderChunkStorage>()
            .init_resource::<TilemapTexturesStorage>()
            .init_resource::<TilemapUniformsStorage>()
            .init_resource::<TilemapBindGroupLayouts>()
            .init_resource::<EntiTilesPipeline>()
            .init_resource::<TilemapBindGroups>()
            .init_resource::<SpecializedRenderPipelines<EntiTilesPipeline>>();

        render_app
            .add_render_command::<Transparent2d, DrawTilemap>()
            .add_render_command::<Transparent2d, DrawTilemapPureColor>();
    }
}
