use std::{mem::size_of, num::NonZeroU64};

use bevy::{
    ecs::{system::Resource, world::FromWorld},
    render::{
        render_resource::{
            BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
            BindingType, BufferBindingType, CachedComputePipelineId, SamplerBindingType,
            ShaderStages, SpecializedComputePipeline,
            StorageTextureAccess, TextureFormat, TextureSampleType, TextureViewDimension,
        },
        renderer::RenderDevice,
    },
};

use super::FogData;

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct MistPipelineKey;

#[derive(Resource)]
pub struct MistPipeline {
    pub mist_layout: BindGroupLayout,
    pub height_texture_layout: BindGroupLayout,
    pub screen_texture_layout: BindGroupLayout,
}

impl FromWorld for MistPipeline {
    fn from_world(world: &mut bevy::prelude::World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let mist_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("mist_layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: NonZeroU64::new(size_of::<FogData>() as u64),
                },
                count: None,
            }],
        });

        let height_texture_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("height_texture_layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: true,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let screen_texture_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("screen_texture_layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::ReadWrite,
                        format: TextureFormat::Rgba8Unorm,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                }],
            });

        Self {
            mist_layout,
            height_texture_layout,
            screen_texture_layout,
        }
    }
}

impl SpecializedComputePipeline for MistPipeline {
    type Key = MistPipelineKey;

    fn specialize(
        &self,
        key: Self::Key,
    ) -> bevy::render::render_resource::ComputePipelineDescriptor {
        todo!()
    }
}
