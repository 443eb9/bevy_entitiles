use bevy::{
    prelude::{FromWorld, Resource},
    render::{
        render_resource::{
            BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
            BlendState, BufferBindingType, ColorTargetState, ColorWrites, Face, FragmentState,
            FrontFace, MultisampleState, PolygonMode, PrimitiveState, PrimitiveTopology,
            RenderPipelineDescriptor, SamplerBindingType, ShaderDefVal, ShaderStages, ShaderType,
            SpecializedRenderPipeline, TextureFormat, TextureSampleType, TextureViewDimension,
            VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
        },
        renderer::RenderDevice,
        texture::BevyDefault,
        view::ViewUniform,
    },
};

use crate::tilemap::tile::TileType;

use super::{uniform::TilemapUniform, TILEMAP_SHADER};

#[derive(Resource)]
pub struct EntiTilesPipeline {
    pub view_layout: BindGroupLayout,
    pub tilemap_uniform_layout: BindGroupLayout,
    pub color_texture_layout: BindGroupLayout,
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct EntiTilesPipelineKey {
    pub msaa: u32,
    pub map_type: TileType,
    pub flip: u32,
    pub is_pure_color: bool,
    pub is_uniform: bool,
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

        let tilemap_uniform_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("tilemap_uniform_layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: Some(TilemapUniform::min_size()),
                    },
                    count: None,
                }],
            });

        let color_texture_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("color_texture_layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
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
            tilemap_uniform_layout,
            color_texture_layout,
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
                    TileType::IsometricDiamond => "ISO_DIAMOND",
                }
            }
            .into(),
        );

        if key.flip & 1u32 != 0 {
            shader_defs.push("FLIP_H".into());
        }
        if key.flip & (1u32 << 1) != 0 {
            shader_defs.push("FLIP_V".into());
        }

        let mut vtx_fmt = vec![
            // position
            VertexFormat::Float32x3,
            // index
            VertexFormat::Float32x2,
            // color
            VertexFormat::Float32x4,
        ];

        if !key.is_pure_color {
            // uv
            vtx_fmt.push(VertexFormat::Float32x2);
        } else {
            shader_defs.push("PURE_COLOR".into());
        }

        if !key.is_uniform {
            // tile_render_size
            vtx_fmt.push(VertexFormat::Float32x2);
            shader_defs.push("NON_UNIFORM".into());
        }

        let vertex_layout =
            VertexBufferLayout::from_vertex_formats(VertexStepMode::Vertex, vtx_fmt);

        let mut layout = vec![
            // group(0)
            self.view_layout.clone(),
            // group(1)
            self.tilemap_uniform_layout.clone(),
        ];

        if !key.is_pure_color {
            // group(2)
            layout.push(self.color_texture_layout.clone());
        }

        RenderPipelineDescriptor {
            label: Some("tilemap_pipeline".into()),
            layout,
            push_constant_ranges: vec![],
            vertex: VertexState {
                shader: TILEMAP_SHADER,
                shader_defs: shader_defs.clone(),
                entry_point: "tilemap_vertex".into(),
                buffers: vec![vertex_layout],
            },
            fragment: Some(FragmentState {
                shader: TILEMAP_SHADER,
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
        }
    }
}
