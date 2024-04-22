use bevy::{
    app::{App, PostUpdate, Update},
    asset::{load_internal_asset, AssetApp},
    core_pipeline::core_2d::Transparent2d,
    ecs::schedule::IntoSystemConfigs,
    prelude::{Handle, Plugin, Shader},
    render::{
        mesh::MeshVertexAttribute,
        render_phase::AddRenderCommand,
        render_resource::{SpecializedRenderPipelines, VertexFormat},
        view::VisibilitySystems,
        ExtractSchedule, Render, RenderApp, RenderSet,
    },
};

use crate::render::{
    binding::{TilemapBindGroupLayouts, TilemapBindGroups},
    buffer::{StandardMaterialUniformBuffer, TilemapStorageBuffers, TilemapUniformBuffer},
    chunk::{ChunkUnload, RenderChunkStorage, UnloadRenderChunk},
    cull::FrustumCulling,
    draw::{DrawTilemap, DrawTilemapWithoutTexture},
    material::{StandardTilemapMaterial, StandardTilemapMaterialInstances},
    resources::TilemapInstances,
    texture::TilemapTexturesStorage,
};

use self::pipeline::EntiTilesPipeline;

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
pub mod resources;
pub mod texture;

pub const SQUARE: Handle<Shader> = Handle::weak_from_u128(54311635145631);
pub const ISOMETRIC: Handle<Shader> = Handle::weak_from_u128(45522415151365135);
pub const HEXAGONAL: Handle<Shader> = Handle::weak_from_u128(341658413214563135);
pub const COMMON: Handle<Shader> = Handle::weak_from_u128(1321023135616351);
pub const TILEMAP_SHADER: Handle<Shader> = Handle::weak_from_u128(89646584153215);

pub const TILEMAP_MESH_ATTR_INDEX: MeshVertexAttribute =
    MeshVertexAttribute::new("GridIndex", 14513156146, VertexFormat::Sint32x4);
pub const TILEMAP_MESH_ATTR_COLOR: MeshVertexAttribute =
    MeshVertexAttribute::new("Color", 85415341854, VertexFormat::Float32x4);
pub const TILEMAP_MESH_ATTR_TEX_INDICES: MeshVertexAttribute =
    MeshVertexAttribute::new("TextureIndex", 186541653135, VertexFormat::Sint32x4);
pub const TILEMAP_MESH_ATTR_FLIP: MeshVertexAttribute =
    MeshVertexAttribute::new("Flip", 7365156123161, VertexFormat::Uint32x4);

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
                .after(bevy::render::view::check_visibility),
        )
        .init_resource::<FrustumCulling>()
        .init_asset::<StandardTilemapMaterial>()
        .register_type::<UnloadRenderChunk>()
        .add_event::<ChunkUnload>();

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
                    extract::extract_changed_tilemaps,
                    extract::extract_tilemaps,
                    extract::extract_tiles,
                    extract::extract_std_materials,
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
                    prepare::prepare_unloaded_chunks,
                    prepare::prepare_despawned_tilemaps,
                    prepare::prepare_despawned_tiles,
                    prepare::prepare_tilemaps,
                    prepare::prepare_tiles,
                    prepare::prepare_std_materials,
                    cull::cull_chunks,
                )
                    .in_set(RenderSet::Prepare),
            )
            .add_systems(Render, queue::queue.in_set(RenderSet::Queue))
            .init_resource::<TilemapTexturesStorage>()
            .init_resource::<TilemapStorageBuffers>()
            .init_resource::<RenderChunkStorage>()
            .init_resource::<TilemapUniformBuffer>()
            .init_resource::<StandardMaterialUniformBuffer>()
            .init_resource::<TilemapBindGroups>()
            .init_resource::<TilemapInstances>()
            .init_resource::<StandardTilemapMaterialInstances>()
            .add_render_command::<Transparent2d, DrawTilemap>()
            .add_render_command::<Transparent2d, DrawTilemapWithoutTexture>();
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);

        render_app
            .init_resource::<TilemapBindGroupLayouts>()
            .init_resource::<EntiTilesPipeline>()
            .init_resource::<SpecializedRenderPipelines<EntiTilesPipeline>>();
    }
}
