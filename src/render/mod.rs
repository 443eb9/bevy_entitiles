use bevy::{
    asset::load_internal_asset,
    prelude::{
        Component, Handle, HandleUntyped, IVec2, Image, IntoSystemConfigs, OrthographicProjection,
        Plugin, Resource, Shader, UVec2, Vec2,
    },
    reflect::TypeUuid,
    render::{
        render_resource::{
            BindGroup, BindGroupLayout, DynamicUniformBuffer, ShaderType,
            SpecializedRenderPipelines,
        },
        ExtractSchedule, Render, RenderApp, RenderSet,
    },
    utils::HashMap,
};

pub mod draw;
pub mod extract;
pub mod pipeline;
pub mod prepare;
pub mod queue;
pub mod texture;

const TILEMAP_SHADER: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 151631653416315);

pub struct EntiTilesRendererPlugin;

impl Plugin for EntiTilesRendererPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {}

    fn finish(&self, _app: &mut bevy::prelude::App) {
        load_internal_asset!(
            _app,
            TILEMAP_SHADER,
            "shaders/tilemap.wgsl",
            Shader::from_wgsl
        );

        let render_app = _app.get_sub_app_mut(RenderApp).unwrap();

        render_app
            .add_systems(ExtractSchedule, extract::extract)
            .add_systems(Render, prepare::prepare.in_set(RenderSet::Prepare))
            .add_systems(Render, queue::queue.in_set(RenderSet::Queue));

        render_app
            .init_resource::<ExtractedData>()
            .init_resource::<UniformData>();

        render_app
            .init_resource::<EntiTilesPipeline>()
            .init_resource::<SpecializedRenderPipelines<EntiTilesPipeline>>();
    }
}

#[derive(Resource)]
pub struct EntiTilesPipeline {
    pub view_layout: BindGroupLayout,
    pub mesh_layout: BindGroupLayout,
    pub texture_layout: BindGroupLayout,
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct EntiTilesPipelineKey {}

#[derive(Component)]
pub struct BindGroups {
    pub tile_data: BindGroup,
    pub tile_textures: HashMap<Handle<Image>, BindGroup>,
}

// Step 1: Extract

pub struct ExtractedTile {
    pub texture_index: u32,
    pub coordinate: IVec2,
}

#[derive(Component)]
pub struct ExtractedView {
    pub projection: OrthographicProjection,
}

#[derive(Resource, Default)]
pub struct ExtractedData {
    pub tiles: Vec<ExtractedTile>,
}

// Step 2: Prepare

#[derive(Resource, Default)]
pub struct UniformData {
    pub tile_data: DynamicUniformBuffer<TileData>,
}

// Step 3: Queue

// Step 4: Draw
