use std::{mem::size_of, num::NonZeroU64, vec};

use bevy::{
    core_pipeline::fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    ecs::{system::Resource, world::FromWorld},
    render::{
        globals::GlobalsUniform,
        render_resource::{
            BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
            BlendState, BufferBindingType, ColorTargetState, ColorWrites, FragmentState,
            MultisampleState, PrimitiveState, RenderPipelineDescriptor, SamplerBindingType,
            ShaderStages, ShaderType, SpecializedRenderPipeline, StorageTextureAccess,
            TextureFormat, TextureSampleType, TextureViewDimension,
        },
        renderer::RenderDevice,
        texture::BevyDefault,
        view::ViewUniform,
    },
};

use crate::post_processing::MIST_SHADER;

use super::FogData;

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct MistPipelineKey {
    pub fog: bool,
    pub height_force_display: bool,
}

#[derive(Resource)]
pub struct MistPipeline {
    pub fog_uniform_layout: BindGroupLayout,
    pub uniforms_layout: BindGroupLayout,
    pub screen_color_texture_layout: BindGroupLayout,
    pub screen_height_texture_layout: BindGroupLayout,
}

impl FromWorld for MistPipeline {
    fn from_world(world: &mut bevy::prelude::World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let fog_uniform_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("fog_uniform_layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: NonZeroU64::new(size_of::<FogData>() as u64),
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: Some(ViewUniform::min_size()),
                        },
                        count: None,
                    },
                ],
            });

        let uniforms_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("uniforms_layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: NonZeroU64::new(size_of::<GlobalsUniform>() as u64),
                },
                count: None,
            }],
        });

        let screen_color_texture_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("screen_color_texture_layout"),
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

        // this should keep the same with the one in `EntitilesPipeline`
        let screen_height_texture_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("screen_height_texture_layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::ReadWrite,
                        format: TextureFormat::Rgba8Unorm,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                }],
            });

        Self {
            fog_uniform_layout,
            uniforms_layout,
            screen_color_texture_layout,
            screen_height_texture_layout,
        }
    }
}

impl SpecializedRenderPipeline for MistPipeline {
    type Key = MistPipelineKey;

    fn specialize(
        &self,
        key: Self::Key,
    ) -> bevy::render::render_resource::RenderPipelineDescriptor {
        let mut layout = vec![
            // group(0)
            self.uniforms_layout.clone(),
            // group(1)
            self.screen_color_texture_layout.clone(),
            // group(2)
            self.screen_height_texture_layout.clone(),
        ];
        let mut shader_defs = vec![];

        if key.fog {
            // group(3)
            layout.push(self.fog_uniform_layout.clone());
            shader_defs.push("FOG".into());
        }

        if key.height_force_display {
            shader_defs.push("HEIGHT_FORCE_DISPLAY".into());
        }

        RenderPipelineDescriptor {
            label: Some("mist_pipeline".into()),
            layout,
            push_constant_ranges: vec![],
            vertex: fullscreen_shader_vertex_state(),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            fragment: Some(FragmentState {
                shader: MIST_SHADER,
                shader_defs,
                entry_point: "mist".into(),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
        }
    }
}
