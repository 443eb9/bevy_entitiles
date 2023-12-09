use bevy::{
    asset::Handle,
    ecs::{entity::Entity, system::Resource, world::FromWorld},
    render::{
        render_resource::{
            BindGroup, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
            BindingType, BufferBindingType, SamplerBindingType, ShaderStages, ShaderType,
            TextureSampleType, TextureViewDimension,
        },
        renderer::RenderDevice,
        texture::Image,
        view::ViewUniform,
    },
    utils::{EntityHashMap, HashMap},
};

use super::uniform::TilemapUniform;

#[derive(Resource, Default)]
pub struct TilemapBindGroups {
    pub tilemap_uniforms: EntityHashMap<Entity, BindGroup>,
    pub colored_textures: HashMap<Handle<Image>, BindGroup>,
}

#[derive(Resource)]
pub struct TilemapBindGroupLayouts {
    pub view_layout: BindGroupLayout,
    pub tilemap_uniform_layout: BindGroupLayout,
    pub color_texture_layout: BindGroupLayout,
}

impl FromWorld for TilemapBindGroupLayouts {
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

        Self {
            view_layout,
            tilemap_uniform_layout,
            color_texture_layout,
        }
    }
}
