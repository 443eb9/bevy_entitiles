use bevy::{
    prelude::{FromWorld, Resource},
    render::{
        render_resource::{
            BindGroupLayout, BlendState, ColorTargetState, ColorWrites, Face, FragmentState,
            FrontFace, MultisampleState, PolygonMode, PrimitiveState, PrimitiveTopology,
            RenderPipelineDescriptor, ShaderDefVal, SpecializedRenderPipeline, TextureFormat,
            VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
        },
        texture::BevyDefault,
    },
};

use crate::tilemap::tile::TileType;

use super::{resources::TilemapBindGroupLayouts, TILEMAP_SHADER};

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
    pub is_pure_color: bool,
    pub is_uniform: bool,
}

impl FromWorld for EntiTilesPipeline {
    fn from_world(world: &mut bevy::prelude::World) -> Self {
        let layouts = world.resource::<TilemapBindGroupLayouts>();
        Self {
            view_layout: layouts.view_layout.clone(),
            tilemap_uniform_layout: layouts.tilemap_uniform_layout.clone(),
            color_texture_layout: layouts.color_texture_layout.clone(),
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
            vtx_fmt.push(VertexFormat::Sint32x4);
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
