use bevy::{
    asset::Handle,
    ecs::{entity::Entity, system::Resource, world::FromWorld},
    math::Vec4,
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
    buffer::{
        TileAnimation, TilemapStorageBuffers, TilemapUniform, TilemapUniformBuffers, UniformBuffer,
    },
    extract::ExtractedTilemap,
    pipeline::EntiTilesPipeline,
    texture::TilemapTexturesStorage,
};

#[derive(Resource, Default)]
pub struct TilemapBindGroups {
    pub tilemap_uniform_buffers: EntityHashMap<Entity, BindGroup>,
    pub tilemap_storage_buffers: EntityHashMap<Entity, BindGroup>,
    pub colored_textures: HashMap<Handle<Image>, BindGroup>,
}

impl TilemapBindGroups {
    pub fn queue_uniform_buffers(
        &mut self,
        tilemap: &ExtractedTilemap,
        render_device: &RenderDevice,
        uniform_buffers: &mut TilemapUniformBuffers,
        entitile_pipeline: &EntiTilesPipeline,
    ) {
        if self.tilemap_uniform_buffers.get(&tilemap.id).is_some() {
            return;
        }

        let Some(uniform_buffer) = uniform_buffers.binding() else {
            return;
        };

        self.tilemap_uniform_buffers.insert(
            tilemap.id,
            render_device.create_bind_group(
                Some("tilemap_uniform_buffers_bind_group"),
                &entitile_pipeline.uniform_buffers_layout,
                &[BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer,
                }],
            ),
        );
    }

    pub fn queue_storage_buffers(
        &mut self,
        tilemap: &ExtractedTilemap,
        render_device: &RenderDevice,
        storage_buffers: &mut TilemapStorageBuffers,
        entitile_pipeline: &EntiTilesPipeline,
    ) {
        if tilemap.texture.is_none() {
            return;
        }

        let (Some(atlas_uvs), Some(anim_seqs)) = (
            storage_buffers.atlas_uvs_binding(tilemap.id),
            storage_buffers.anim_seqs_binding(tilemap.id),
        ) else {
            return;
        };

        self.tilemap_storage_buffers.insert(
            tilemap.id,
            render_device.create_bind_group(
                Some("tilemap_storage_bind_group"),
                &entitile_pipeline.storage_buffers_layout,
                &[
                    BindGroupEntry {
                        binding: 0,
                        resource: atlas_uvs,
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: anim_seqs,
                    },
                ],
            ),
        );
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
    pub tilemap_uniforms_layout: BindGroupLayout,
    pub tilemap_storage_layout: BindGroupLayout,
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

        let tilemap_uniforms_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("tilemap_uniforms_layout"),
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

        let tilemap_storage_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("tilemap_storage_layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::VERTEX,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: Some(Vec4::min_size()),
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::VERTEX,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: Some(TileAnimation::min_size()),
                        },
                        count: None,
                    },
                ],
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
            tilemap_uniforms_layout,
            tilemap_storage_layout,
            color_texture_layout,
        }
    }
}
