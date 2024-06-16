use bevy::{
    asset::{AssetId, Handle},
    ecs::{component::Component, entity::EntityHashMap, system::Resource},
    log::error,
    render::{
        render_asset::RenderAssets,
        render_resource::{BindGroup, BindGroupEntries, BindGroupLayout},
        renderer::RenderDevice,
        texture::{FallbackImage, GpuImage},
    },
    utils::HashMap,
};

use crate::{
    render::{
        buffer::{
            PerTilemapBuffersStorage, TilemapAnimationBuffer, TilemapUniformBuffer, UniformBuffer,
        },
        extract::ExtractedTilemap,
        material::TilemapMaterial,
        pipeline::EntiTilesPipeline,
        resources::ExtractedTilemapMaterials,
        texture::TilemapTexturesStorage,
    },
    tilemap::map::TilemapTextures,
};

#[cfg(feature = "atlas")]
use crate::render::buffer::TilemapTextureDescriptorBuffer;

#[derive(Component)]
pub struct TilemapViewBindGroup {
    pub value: BindGroup,
}

#[derive(Resource)]
pub struct TilemapBindGroups<M: TilemapMaterial> {
    pub uniform_buffer: Option<BindGroup>,
    pub storage_buffers: EntityHashMap<BindGroup>,
    pub textures: HashMap<Handle<TilemapTextures>, BindGroup>,
    pub materials: HashMap<AssetId<M>, BindGroup>,
}

impl<M: TilemapMaterial> Default for TilemapBindGroups<M> {
    fn default() -> Self {
        Self {
            uniform_buffer: Default::default(),
            storage_buffers: Default::default(),
            textures: Default::default(),
            materials: Default::default(),
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

        self.uniform_buffer = Some(render_device.create_bind_group(
            Some("tilemap_uniform_buffers_bind_group"),
            &entitiles_pipeline.uniform_buffers_layout,
            &BindGroupEntries::single(uniform_buffer),
        ));
    }

    pub fn bind_tilemap_storage_buffers(
        &mut self,
        render_device: &RenderDevice,
        animation_buffers: &mut TilemapAnimationBuffer,
        entitiles_pipeline: &EntiTilesPipeline<M>,
        #[cfg(feature = "atlas")] texture_desc_buffers: &mut TilemapTextureDescriptorBuffer,
    ) {
        let anim_bindings = animation_buffers.bindings();
        #[cfg(feature = "atlas")]
        let tex_desc_bindings = texture_desc_buffers.bindings();

        for tilemap in anim_bindings.keys() {
            let Some(anim) = anim_bindings.get(tilemap) else {
                error!("It seems that there are some tilemaps that have textures but no `TilemapAnimations`, which is not allowed");
                return;
            };

            #[cfg(feature = "atlas")]
            let Some(tex_desc) = tex_desc_bindings.get(tilemap) else {
                error!("It seems that there are some tilemaps that have textures but no `TilemapAnimations`, which is not allowed");
                return;
            };

            #[cfg(not(feature = "atlas"))]
            self.storage_buffers.insert(
                *tilemap,
                render_device.create_bind_group(
                    "tilemap_storage_buffers_bind_group",
                    &entitiles_pipeline.storage_buffers_layout,
                    &BindGroupEntries::single(anim.clone()),
                ),
            );

            #[cfg(feature = "atlas")]
            self.storage_buffers.insert(
                *tilemap,
                render_device.create_bind_group(
                    "tilemap_storage_buffers_bind_group",
                    &entitiles_pipeline.storage_buffers_layout,
                    &BindGroupEntries::sequential((anim.clone(), tex_desc.clone())),
                ),
            );
        }
    }

    pub fn prepare_material_bind_groups(
        &mut self,
        layout: &BindGroupLayout,
        render_device: &RenderDevice,
        images: &RenderAssets<GpuImage>,
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
                self.materials.insert(*id, bind_group.bind_group);
            });
    }

    /// Returns is_pure_color
    pub fn queue_textures(
        &mut self,
        tilemap: &ExtractedTilemap<M>,
        render_device: &RenderDevice,
        textures_storage: &TilemapTexturesStorage,
        entitiles_pipeline: &EntiTilesPipeline<M>,
    ) -> bool {
        let Some(tilemap_texture) = &tilemap.texture else {
            return true;
        };

        let Some(texture) = textures_storage.get_texture(tilemap_texture) else {
            return !textures_storage.contains(tilemap_texture);
        };

        if !self.textures.contains_key(tilemap_texture) {
            self.textures.insert(
                tilemap_texture.clone_weak(),
                render_device.create_bind_group(
                    Some("color_texture_bind_group"),
                    &entitiles_pipeline.texture_layout,
                    &BindGroupEntries::sequential((&texture.texture_view, &texture.sampler)),
                ),
            );
        }

        false
    }
}
