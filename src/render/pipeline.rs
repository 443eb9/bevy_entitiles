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

use super::{binding::TilemapBindGroupLayouts, TILEMAP_SHADER};

#[derive(Resource)]
pub struct EntiTilesPipeline {
    pub view_layout: BindGroupLayout,
    pub uniform_buffers_layout: BindGroupLayout,
    pub storage_buffers_layout: BindGroupLayout,
    pub color_texture_layout: BindGroupLayout,
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct EntiTilesPipelineKey {
    pub msaa: u32,
    pub map_type: TileType,
    pub is_pure_color: bool,
}

impl FromWorld for EntiTilesPipeline {
    fn from_world(world: &mut bevy::prelude::World) -> Self {
        let layouts = world.resource::<TilemapBindGroupLayouts>();
        Self {
            view_layout: layouts.view_layout.clone(),
            uniform_buffers_layout: layouts.tilemap_uniforms_layout.clone(),
            storage_buffers_layout: layouts.tilemap_storage_layout.clone(),
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
                    TileType::Isometric => "ISOMETRIC",
                    TileType::Hexagonal(_) => "HEXAGONAL",
                }
            }
            .into(),
        );

        let mut vtx_fmt = vec![
            // position
            VertexFormat::Float32x3,
            // index + is_animated
            VertexFormat::Uint32x3,
            // color
            VertexFormat::Float32x4,
        ];

        if key.is_pure_color {
            shader_defs.push("PURE_COLOR".into());
        } else {
            // texture_indices
            vtx_fmt.push(VertexFormat::Sint32x4);
            // flip
            vtx_fmt.push(VertexFormat::Uint32x4);
        }

        let vertex_layout =
            VertexBufferLayout::from_vertex_formats(VertexStepMode::Vertex, vtx_fmt);

        let mut layout = vec![
            // group(0)
            self.view_layout.clone(),
            // group(1)
            self.uniform_buffers_layout.clone(),
        ];

        if !key.is_pure_color {
            // // group(2)
            // layout.push(self.storage_buffers_layout.clone());
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
                    blend: Some(BlendState::PREMULTIPLIED_ALPHA_BLENDING),
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
