use std::{mem::size_of, num::NonZeroU64};

use bevy::{
    ecs::{system::Resource, world::FromWorld},
    render::{
        render_resource::{
            BindGroup, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
            BindingType, BufferBindingType, CachedComputePipelineId, SamplerBindingType, ShaderStages, StorageTextureAccess, TextureFormat,
            TextureSampleType, TextureViewDimension,
        },
        renderer::RenderDevice,
    },
};

use super::mist::CloudsData;

#[derive(Resource)]
pub struct MistPipeline {
    pub cached_id: Option<CachedComputePipelineId>,
    pub mist_layout: BindGroupLayout,
    pub screen_texture_layout: BindGroupLayout,
    pub mist_group: Option<BindGroup>,
    pub screen_texture_group: Option<BindGroup>,
}

impl FromWorld for MistPipeline {
    fn from_world(world: &mut bevy::prelude::World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let screen_texture_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("screen_texture_group_layout"),
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

        let mist_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("mist_bind_group_layout"),
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
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: NonZeroU64::new(size_of::<CloudsData>() as u64),
                    },
                    count: None,
                },
            ],
        });

        Self {
            cached_id: None,
            mist_layout,
            screen_texture_layout,
            mist_group: None,
            screen_texture_group: None,
        }
    }
}
