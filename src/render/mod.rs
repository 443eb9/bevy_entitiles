use bevy::{
    app::{App, PostUpdate, Update},
    asset::load_internal_asset,
    ecs::schedule::IntoSystemConfigs,
    prelude::{Handle, Plugin, Shader},
    render::{
        extract_instances::ExtractInstancesPlugin, mesh::MeshVertexAttribute,
        render_asset::RenderAssetPlugin, render_resource::VertexFormat, view::VisibilitySystems,
        ExtractSchedule, Render, RenderApp, RenderSet,
    },
};

use crate::{
    render::{
        buffer::TilemapBuffers,
        chunk::{ChunkUnload, RenderChunkSort, RenderChunkStorage, UnloadRenderChunk},
        cull::FrustumCulling,
        extract::ExtractedTilemap,
        texture::TilemapTexturesStorage,
    },
    tilemap::map::TilemapTextures,
};

#[cfg(feature = "baking")]
pub mod bake;
pub mod binding;
pub mod buffer;
pub mod chunk;
pub mod cull;
pub mod draw;
pub mod extract;
pub mod material;
pub mod pipeline;
pub mod prepare;
pub mod queue;
pub mod texture;

pub const SQUARE: Handle<Shader> = Handle::weak_from_u128(54311635145631);
pub const ISOMETRIC: Handle<Shader> = Handle::weak_from_u128(45522415151365135);
pub const HEXAGONAL: Handle<Shader> = Handle::weak_from_u128(341658413214563135);
pub const COMMON: Handle<Shader> = Handle::weak_from_u128(1321023135616351);
pub const TILEMAP_SHADER: Handle<Shader> = Handle::weak_from_u128(89646584153215);

pub const TILEMAP_MESH_ATTR_INDEX: MeshVertexAttribute =
    MeshVertexAttribute::new("GridIndex", 51541631, VertexFormat::Sint32x4);
pub const TILEMAP_MESH_ATTR_COLOR: MeshVertexAttribute =
    MeshVertexAttribute::new("Color", 51541632, VertexFormat::Float32x4);
pub const TILEMAP_MESH_ATTR_ATLAS_INDICES: MeshVertexAttribute =
    MeshVertexAttribute::new("AtlasIndex", 51541633, VertexFormat::Sint32x4);
#[cfg(feature = "atlas")]
pub const TILEMAP_MESH_ATTR_TEX_INDICES: MeshVertexAttribute =
    MeshVertexAttribute::new("TextureIndex", 51541634, VertexFormat::Sint32x4);

#[derive(Default)]
pub struct EntiTilesRendererPlugin;

impl Plugin for EntiTilesRendererPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(app, SQUARE, "shaders/square.wgsl", Shader::from_wgsl);
        load_internal_asset!(app, ISOMETRIC, "shaders/isometric.wgsl", Shader::from_wgsl);
        load_internal_asset!(app, HEXAGONAL, "shaders/hexagonal.wgsl", Shader::from_wgsl);
        load_internal_asset!(app, COMMON, "shaders/common.wgsl", Shader::from_wgsl);

        load_internal_asset!(
            app,
            TILEMAP_SHADER,
            "shaders/tilemap.wgsl",
            Shader::from_wgsl
        );

        app.add_systems(
            Update,
            (
                texture::set_texture_usage,
                #[cfg(feature = "baking")]
                bake::tilemap_baker,
            ),
        )
        .add_systems(
            PostUpdate,
            cull::cull_tilemaps
                .in_set(VisibilitySystems::CheckVisibility)
                .after(bevy::render::view::check_visibility::<()>),
        )
        .init_resource::<FrustumCulling>()
        .init_resource::<RenderChunkSort>()
        .register_type::<UnloadRenderChunk>()
        .add_event::<ChunkUnload>()
        .add_plugins((
            RenderAssetPlugin::<TilemapTextures>::default(),
            ExtractInstancesPlugin::<ExtractedTilemap>::new(),
        ));

        #[cfg(feature = "baking")]
        {
            use bake::{BakedTilemap, TilemapBaker};

            app.register_type::<TilemapBaker>()
                .register_type::<BakedTilemap>();
        }

        let render_app = app.sub_app_mut(RenderApp);

        render_app
            .add_systems(
                ExtractSchedule,
                (
                    extract::extract_tiles,
                    extract::extract_view,
                    extract::extract_unloaded_chunks,
                    extract::extract_resources,
                    extract::extract_despawned_tilemaps,
                    extract::extract_despawned_tiles,
                ),
            )
            .add_systems(
                Render,
                (
                    texture::schedule_tilemap_texture_preparation,
                    texture::prepare_tilemap_textures,
                    texture::queue_tilemap_textures,
                    buffer::prepare_tilemap_buffers,
                    cull::cull_chunks,
                )
                    .chain()
                    .in_set(RenderSet::PrepareResources),
            )
            .init_resource::<RenderChunkSort>()
            .init_resource::<RenderChunkStorage>()
            .init_resource::<TilemapTexturesStorage>()
            .init_resource::<TilemapBuffers>();
    }
}
