use bevy::{
    asset::Handle,
    ecs::{entity::Entity, system::Resource, world::FromWorld},
    render::{
        render_resource::{
            BindGroup, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
            BindGroupLayoutEntry, BindingResource, BindingType, BufferBindingType,
            SamplerBindingType, ShaderStages, ShaderType, TextureSampleType, TextureViewDimension,
        },
        renderer::RenderDevice,
        texture::Image,
        view::ViewUniform,
    },
    utils::{EntityHashMap, HashMap},
};

use super::{
    extract::ExtractedTilemap,
    pipeline::EntiTilesPipeline,
    texture::TilemapTexturesStorage,
    uniform::{TilemapUniform, TilemapUniformsStorage, UniformsStorage},
};

#[derive(Resource, Default)]
pub struct TilemapBindGroups {
    pub tilemap_uniforms: EntityHashMap<Entity, BindGroup>,
    pub colored_textures: HashMap<Handle<Image>, BindGroup>,
}

impl TilemapBindGroups {
    pub fn queue_uniforms(
        &mut self,
        tilemap: &ExtractedTilemap,
        render_device: &RenderDevice,
        tilemap_uniform_storage: &mut TilemapUniformsStorage,
        entitile_pipeline: &EntiTilesPipeline,
    ) {
        if self.tilemap_uniforms.get(&tilemap.id).is_none() {
            let Some(resource) = tilemap_uniform_storage.binding() else {
                return;
            };

            self.tilemap_uniforms.insert(
                tilemap.id,
                render_device.create_bind_group(
                    Some("tilemap_uniform_bind_group"),
                    &entitile_pipeline.tilemap_uniform_layout,
                    &[BindGroupEntry {
                        binding: 0,
                        resource,
                    }],
                ),
            );
        }
    }

    /// Returns (is_pure_color, is_uniform)
    pub fn queue_textures(
        &mut self,
        tilemap: &ExtractedTilemap,
        render_device: &RenderDevice,
        textures_storage: &TilemapTexturesStorage,
        entitile_pipeline: &EntiTilesPipeline,
    ) -> (bool, bool) {
        if let Some(tilemap_texture) = &tilemap.texture {
            let Some(texture) = textures_storage.get(tilemap_texture.handle()) else {
                return (true, true);
            };

            if !self.colored_textures.contains_key(tilemap_texture.handle()) {
                self.colored_textures.insert(
                    tilemap_texture.clone_weak(),
                    render_device.create_bind_group(
                        Some("color_texture_bind_group"),
                        &entitile_pipeline.color_texture_layout,
                        &[
                            BindGroupEntry {
                                binding: 0,
                                resource: BindingResource::TextureView(&texture.texture_view),
                            },
                            BindGroupEntry {
                                binding: 1,
                                resource: BindingResource::Sampler(&texture.sampler),
                            },
                        ],
                    ),
                );
            }

            (false, tilemap_texture.desc().is_uniform)
        } else {
            (true, true)
        }
    }
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
