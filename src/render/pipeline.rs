use std::marker::PhantomData;

use bevy::{
    asset::{AssetServer, Handle},
    ecs::world::World,
    prelude::{FromWorld, Resource},
    render::{
        render_resource::{
            BindGroupLayout, BindGroupLayoutEntries, BlendState, ColorTargetState, ColorWrites,
            Face, FragmentState, FrontFace, MultisampleState, PolygonMode, PrimitiveState,
            PrimitiveTopology, RenderPipelineDescriptor, SamplerBindingType, Shader, ShaderDefVal,
            ShaderRef, ShaderStages, SpecializedRenderPipeline, TextureFormat, TextureSampleType,
            VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
        },
        renderer::RenderDevice,
        texture::BevyDefault,
        view::ViewUniform,
    },
};

use crate::{
    render::{buffer::TilemapUniform, material::TilemapMaterial},
    tilemap::map::TilemapType,
};

#[cfg(feature = "atlas")]
use crate::render::buffer::GpuTilemapTextureDescriptor;

use bevy::render::render_resource::binding_types as binding;

#[derive(Resource)]
pub struct EntiTilesPipeline<M: TilemapMaterial> {
    pub uniform_buffers_layout: BindGroupLayout,
    pub texture_layout: BindGroupLayout,
    pub storage_buffers_layout: BindGroupLayout,
    pub material_layout: BindGroupLayout,
    pub vertex_shader: Handle<Shader>,
    pub fragment_shader: Handle<Shader>,
    pub marker: PhantomData<M>,
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct EntiTilesPipelineKey {
    pub msaa: u32,
    pub map_type: TilemapType,
    pub is_pure_color: bool,
}

impl<M: TilemapMaterial> FromWorld for EntiTilesPipeline<M> {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let render_device = world.get_resource::<RenderDevice>().unwrap();

        let uniform_buffers_layout = render_device.create_bind_group_layout(
            "uniform_buffers_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::VERTEX_FRAGMENT,
                (
                    binding::uniform_buffer::<TilemapUniform>(true),
                    binding::uniform_buffer::<ViewUniform>(true),
                ),
            ),
        );

        #[cfg(not(feature = "atlas"))]
        let storage_buffers_layout = render_device.create_bind_group_layout(
            "animation_buffer_layout",
            &BindGroupLayoutEntries::single(
                ShaderStages::VERTEX_FRAGMENT,
                binding::storage_buffer_read_only::<i32>(false),
            ),
        );

        #[cfg(feature = "atlas")]
        let storage_buffers_layout = render_device.create_bind_group_layout(
            "animation_buffer_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::VERTEX_FRAGMENT,
                (
                    binding::storage_buffer_read_only::<i32>(false),
                    binding::storage_buffer_read_only::<Vec<GpuTilemapTextureDescriptor>>(false),
                ),
            ),
        );

        let texture_layout = render_device.create_bind_group_layout(
            "textured_tilemap_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    binding::texture_2d_array(TextureSampleType::Float { filterable: true }),
                    binding::sampler(SamplerBindingType::Filtering),
                ),
            ),
        );

        Self {
            uniform_buffers_layout,
            texture_layout,
            storage_buffers_layout,
            material_layout: M::bind_group_layout(render_device),
            vertex_shader: match M::vertex_shader() {
                ShaderRef::Default => panic!("You must provide a valid custom vertex shader!"),
                ShaderRef::Handle(handle) => handle,
                ShaderRef::Path(path) => asset_server.load(path),
            },
            fragment_shader: match M::fragment_shader() {
                ShaderRef::Default => {
                    panic!("You must provide a valid custom fragment shader!")
                }
                ShaderRef::Handle(handle) => handle,
                ShaderRef::Path(path) => asset_server.load(path),
            },
            marker: PhantomData,
        }
    }
}

impl<M: TilemapMaterial> SpecializedRenderPipeline for EntiTilesPipeline<M> {
    type Key = EntiTilesPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        let mut shader_defs: Vec<ShaderDefVal> = vec![];
        shader_defs.push(
            {
                match key.map_type {
                    TilemapType::Square => "SQUARE",
                    TilemapType::Isometric => "ISOMETRIC",
                    TilemapType::Hexagonal(_) => "HEXAGONAL",
                }
            }
            .into(),
        );
        #[cfg(feature = "atlas")]
        shader_defs.push("ATLAS".into());

        let mut vtx_fmt = vec![
            // position
            VertexFormat::Float32x3,
            // index + anim_start + anim_len
            VertexFormat::Sint32x4,
            // color
            VertexFormat::Float32x4,
        ];

        if key.is_pure_color {
            shader_defs.push("PURE_COLOR".into());
        } else {
            // atlas indices
            vtx_fmt.push(VertexFormat::Sint32x4);

            #[cfg(feature = "atlas")]
            // texture_indices
            vtx_fmt.push(VertexFormat::Sint32x4);
        }

        let vertex_layout =
            VertexBufferLayout::from_vertex_formats(VertexStepMode::Vertex, vtx_fmt);

        let mut layout = vec![
            // group(0)
            self.uniform_buffers_layout.clone(),
            // group(1)
            self.material_layout.clone(),
        ];

        if !key.is_pure_color {
            // group(2)
            layout.push(self.texture_layout.clone());
            // group(3)
            layout.push(self.storage_buffers_layout.clone());
        }

        let mut desc = RenderPipelineDescriptor {
            label: Some("tilemap_pipeline".into()),
            layout,
            push_constant_ranges: vec![],
            vertex: VertexState {
                shader: self.vertex_shader.clone(),
                shader_defs: shader_defs.clone(),
                entry_point: "tilemap_vertex".into(),
                buffers: vec![vertex_layout],
            },
            fragment: Some(FragmentState {
                shader: self.fragment_shader.clone(),
                shader_defs: shader_defs.clone(),
                entry_point: "tilemap_fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Cw,
                cull_mode: Some(Face::Back),
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: key.msaa,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        };

        M::specialize(&mut desc);

        desc
    }
}
