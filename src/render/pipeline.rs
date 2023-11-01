use bevy::{
    prelude::{FromWorld, Resource},
    render::{
        globals::GlobalsUniform,
        render_resource::{
            BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
            BufferBindingType, ColorTargetState, ColorWrites, Face, FragmentState, FrontFace,
            MultisampleState, PolygonMode, PrimitiveState, PrimitiveTopology,
            RenderPipelineDescriptor, SamplerBindingType, ShaderDefVal, ShaderStages, ShaderType,
            SpecializedRenderPipeline, TextureFormat, TextureSampleType, TextureViewDimension,
            VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
        },
        renderer::RenderDevice,
        texture::BevyDefault,
        view::ViewUniform,
    },
};

use crate::tilemap::TileType;

use super::TILEMAP_SHADER;

#[derive(Resource)]
pub struct EntiTilesPipeline {
    pub view_layout: BindGroupLayout,
    pub tilemap_data_layout: BindGroupLayout,
    pub texture_layout: BindGroupLayout,
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct EntiTilesPipelineKey {
    pub msaa: u32,
    pub map_type: TileType,
}

impl FromWorld for EntiTilesPipeline {
    fn from_world(world: &mut bevy::prelude::World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let view_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("tilemap_view_layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: Some(ViewUniform::min_size()),
                },
                count: None,
            }],
        });

        let tilemap_data_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("tilemap_data_layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: Some(GlobalsUniform::min_size()),
                    },
                    count: None,
                }],
            });

        let texture_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("tilemap_texture_layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        EntiTilesPipeline {
            view_layout,
            tilemap_data_layout,
            texture_layout,
        }
    }
}

impl SpecializedRenderPipeline for EntiTilesPipeline {
    type Key = EntiTilesPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        let mut shader_defs: Vec<ShaderDefVal> = vec![];
        shader_defs.push(
            {
                match key.map_type {
                    TileType::Square => "SQUARE",
                }
            }
            .into(),
        );

        let vertex_layout = VertexBufferLayout::from_vertex_formats(
            VertexStepMode::Vertex,
            vec![
                // position
                VertexFormat::Float32x3,
                // texture_index
                VertexFormat::Uint32,
                // grid_index
                VertexFormat::Uint32x2,
            ],
        );

        RenderPipelineDescriptor {
            label: Some("tilemap_pipeline".into()),
            layout: vec![self.view_layout.clone(), self.texture_layout.clone()],
            push_constant_ranges: vec![],
            vertex: VertexState {
                shader: TILEMAP_SHADER.typed(),
                shader_defs,
                entry_point: "tilemap_vertex".into(),
                buffers: vec![vertex_layout],
            },
            fragment: Some(FragmentState {
                shader: TILEMAP_SHADER.typed(),
                shader_defs: vec![],
                entry_point: "tilemap_fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
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
        }
    }
}

pub struct TilemapPipelineKey {}
