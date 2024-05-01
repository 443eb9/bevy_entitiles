use bevy::{
    asset::{AssetId, Handle},
    ecs::{component::Component, entity::EntityHashMap, system::Resource, world::FromWorld},
    render::{
        render_asset::RenderAssets,
        render_resource::{
            BindGroup, BindGroupEntries, BindGroupLayout, BindGroupLayoutEntries,
            SamplerBindingType, ShaderStages, TextureSampleType,
        },
        renderer::RenderDevice,
        texture::{FallbackImage, Image},
        view::ViewUniform,
    },
    utils::HashMap,
};

use super::{
    buffer::{
        PerTilemapBuffersStorage, TilemapStorageBuffers, TilemapUniform, TilemapUniformBuffer,
        UniformBuffer,
    },
    extract::ExtractedTilemap,
    material::TilemapMaterial,
    pipeline::EntiTilesPipeline,
    resources::ExtractedTilemapMaterials,
    texture::TilemapTexturesStorage,
};

use bevy::render::render_resource::binding_types as binding;

#[derive(Component)]
pub struct TilemapViewBindGroup {
    pub value: BindGroup,
}

#[derive(Resource)]
pub struct TilemapBindGroups<M: TilemapMaterial> {
    pub tilemap_uniform_buffer: Option<BindGroup>,
    pub tilemap_storage_buffers: EntityHashMap<BindGroup>,
    pub colored_textures: HashMap<Handle<Image>, BindGroup>,
    pub material_bind_groups: HashMap<AssetId<M>, BindGroup>,
}

impl<M: TilemapMaterial> Default for TilemapBindGroups<M> {
    fn default() -> Self {
        Self {
            tilemap_uniform_buffer: Default::default(),
            tilemap_storage_buffers: Default::default(),
            colored_textures: Default::default(),
            material_bind_groups: Default::default(),
        }
    }
}

impl<M: TilemapMaterial> TilemapBindGroups<M> {
    pub fn bind_uniform_buffers(
        &mut self,
        render_device: &RenderDevice,
        uniform_buffers: &mut TilemapUniformBuffer<M>,
        entitiles_pipeline: &EntiTilesPipeline<M>,
    ) {
        let Some(uniform_buffer) = uniform_buffers.binding() else {
            return;
        };

        self.tilemap_uniform_buffer = Some(render_device.create_bind_group(
            Some("tilemap_uniform_buffers_bind_group"),
            &entitiles_pipeline.uniform_buffers_layout,
            &BindGroupEntries::single(uniform_buffer),
        ));
    }

    pub fn bind_storage_buffers(
        &mut self,
        render_device: &RenderDevice,
        storage_buffers: &mut TilemapStorageBuffers,
        entitiles_pipeline: &EntiTilesPipeline<M>,
    ) {
        storage_buffers
            .bindings()
            .into_iter()
            .for_each(|(tilemap, resource)| {
                self.tilemap_storage_buffers.insert(
                    tilemap,
                    render_device.create_bind_group(
                        Some("tilemap_storage_bind_group"),
                        &entitiles_pipeline.storage_buffers_layout,
                        &BindGroupEntries::single(resource),
                    ),
                );
            });
    }

    pub fn prepare_material_bind_groups(
        &mut self,
        layout: &BindGroupLayout,
        render_device: &RenderDevice,
        images: &RenderAssets<Image>,
        fallback_image: &FallbackImage,
        extracted_materials: &ExtractedTilemapMaterials<M>,
    ) {
        extracted_materials
            .changed
            .iter()
            .for_each(|(id, material)| {
                let bind_group = material
                    .as_bind_group(layout, render_device, images, fallback_image)
                    .unwrap();
                self.material_bind_groups.insert(*id, bind_group.bind_group);
            });
    }

    /// Returns is_pure_color
    pub fn queue_textures(
        &mut self,
        tilemap: &ExtractedTilemap<M>,
        render_device: &RenderDevice,
        textures_storage: &TilemapTexturesStorage,
        entitile_pipeline: &EntiTilesPipeline<M>,
    ) -> bool {
        let Some(tilemap_texture) = &tilemap.texture else {
            return true;
        };

        let Some(texture) = textures_storage.get_texture(tilemap_texture.handle()) else {
            return !textures_storage.contains(tilemap_texture.handle());
        };

        if !self.colored_textures.contains_key(tilemap_texture.handle()) {
            self.colored_textures.insert(
                tilemap_texture.clone_weak(),
                render_device.create_bind_group(
                    Some("color_texture_bind_group"),
                    &entitile_pipeline.color_texture_layout,
                    &BindGroupEntries::sequential((&texture.texture_view, &texture.sampler)),
                ),
            );
        }

        false
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
        let view_layout = render_device.create_bind_group_layout(
            "tilemap_view_layout",
            &BindGroupLayoutEntries::single(
                ShaderStages::VERTEX_FRAGMENT,
                binding::uniform_buffer::<ViewUniform>(true),
            ),
        );

        let tilemap_uniforms_layout = render_device.create_bind_group_layout(
            "tilemap_uniforms_layout",
            &BindGroupLayoutEntries::single(
                ShaderStages::VERTEX_FRAGMENT,
                binding::uniform_buffer::<TilemapUniform>(true),
            ),
        );

        let tilemap_storage_layout = render_device.create_bind_group_layout(
            "tilemap_storage_layout",
            &BindGroupLayoutEntries::single(
                ShaderStages::VERTEX,
                binding::storage_buffer_read_only::<i32>(false),
            ),
        );

        #[cfg(not(feature = "atlas"))]
        let color_texture_layout = render_device.create_bind_group_layout(
            "color_texture_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    binding::texture_2d_array(TextureSampleType::Float { filterable: true }),
                    binding::sampler(SamplerBindingType::Filtering),
                ),
            ),
        );

        #[cfg(feature = "atlas")]
        let color_texture_layout = render_device.create_bind_group_layout(
            "color_texture_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    binding::texture_2d(TextureSampleType::Float { filterable: true }),
                    binding::sampler(SamplerBindingType::Filtering),
                ),
            ),
        );

        Self {
            view_layout,
            tilemap_uniforms_layout,
            tilemap_storage_layout,
            color_texture_layout,
        }
    }
}
