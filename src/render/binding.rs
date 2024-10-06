use bevy::{
    asset::{AssetId, Handle},
    ecs::{component::Component, entity::EntityHashMap, system::Resource},
    log::error,
    prelude::{Entity, Query, Res, ResMut, With},
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
        buffer::TilemapBuffers,
        extract::{ExtractedTilemap, TilemapInstance},
        material::TilemapMaterial,
        pipeline::EntiTilesPipeline,
        resources::{ExtractedTilemapMaterials, TilemapInstances},
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

#[derive(Resource, Default)]
pub struct TilemapBindGroups<M: TilemapMaterial> {
    pub uniform_buffer: Option<BindGroup>,
    pub storage_buffers: EntityHashMap<BindGroup>,
    pub textures: HashMap<Handle<TilemapTextures>, BindGroup>,
    pub materials: HashMap<AssetId<M>, BindGroup>,
}

pub fn bind_tilemap_buffers<M: TilemapMaterial>(
    render_device: Res<RenderDevice>,
    entitiles_pipeline: Res<EntiTilesPipeline<M>>,
    // #[cfg(feature = "atlas")] texture_desc_buffers: &mut TilemapTextureDescriptorBuffer,
    tilemaps_query: Query<Entity, With<TilemapInstance>>,
    tilemap_buffers: Res<TilemapBuffers>,
    mut bind_groups: ResMut<TilemapBindGroups<M>>,
) {
    if let Some(uniform_buffer) = tilemap_buffers.shared.uniform.binding() {
        bind_groups.uniform_buffer = Some(render_device.create_bind_group(
            Some("tilemap_uniform_buffers_bind_group"),
            &entitiles_pipeline.uniform_buffers_layout,
            &BindGroupEntries::single(uniform_buffer),
        ));
    }

    for tilemap in &tilemaps_query {
        let Some(buffers) = tilemap_buffers.unshared.get(&tilemap) else {
            continue;
        };

        let Some(anim) = buffers.animation.binding() else {
            continue;
        };

        bind_groups.storage_buffers.insert(
            tilemap,
            render_device.create_bind_group(
                "tilemap_storage_buffers_bind_group",
                &entitiles_pipeline.storage_buffers_layout,
                &BindGroupEntries::single(anim),
            ),
        );
    }

    // let anim_bindings = animation_buffers.bindings();
    // #[cfg(feature = "atlas")]
    // let tex_desc_bindings = texture_desc_buffers.bindings();

    // for tilemap in anim_bindings.keys() {
    //     let Some(anim) = anim_bindings.get(tilemap) else {
    //         error!("It seems that there are some tilemaps that have textures but no `TilemapAnimations`, which is not allowed");
    //         return;
    //     };

    //     #[cfg(feature = "atlas")]
    //     let Some(tex_desc) = tex_desc_bindings.get(tilemap) else {
    //         error!("It seems that there are some tilemaps that have textures but no `TilemapAnimations`, which is not allowed");
    //         return;
    //     };

    //     #[cfg(not(feature = "atlas"))]
    //     self.storage_buffers.insert(
    //         *tilemap,
    //         render_device.create_bind_group(
    //             "tilemap_storage_buffers_bind_group",
    //             &entitiles_pipeline.storage_buffers_layout,
    //             &BindGroupEntries::single(anim.clone()),
    //         ),
    //     );

    //     #[cfg(feature = "atlas")]
    //     self.storage_buffers.insert(
    //         *tilemap,
    //         render_device.create_bind_group(
    //             "tilemap_storage_buffers_bind_group",
    //             &entitiles_pipeline.storage_buffers_layout,
    //             &BindGroupEntries::sequential((anim.clone(), tex_desc.clone())),
    //         ),
    //     );
    // }
}

pub fn bind_materials<M: TilemapMaterial>(
    pipeline: Res<EntiTilesPipeline<M>>,
    render_device: Res<RenderDevice>,
    images: Res<RenderAssets<GpuImage>>,
    fallback_image: Res<FallbackImage>,
    extracted_materials: Res<ExtractedTilemapMaterials<M>>,
    mut bind_groups: ResMut<TilemapBindGroups<M>>,
) {
    extracted_materials
        .changed
        .iter()
        .for_each(|(id, material)| {
            let bind_group = material
                .as_bind_group(
                    &pipeline.material_layout,
                    &render_device,
                    &images,
                    &fallback_image,
                )
                .unwrap();
            bind_groups.materials.insert(*id, bind_group.bind_group);
        });
}

pub fn bind_textures<M: TilemapMaterial>(
    tilemap_instances: Res<TilemapInstances<M>>,
    render_device: Res<RenderDevice>,
    textures_storage: Res<TilemapTexturesStorage>,
    entitiles_pipeline: Res<EntiTilesPipeline<M>>,
    mut bind_groups: ResMut<TilemapBindGroups<M>>,
) {
    for tilemap in tilemap_instances.0.values() {
        let Some((handle, texture)) = tilemap
            .texture
            .as_ref()
            .and_then(|h| textures_storage.get_texture(h).map(|t| (h, t)))
        else {
            continue;
        };

        if !bind_groups.textures.contains_key(handle) {
            bind_groups.textures.insert(
                handle.clone(),
                render_device.create_bind_group(
                    Some("color_texture_bind_group"),
                    &entitiles_pipeline.texture_layout,
                    &BindGroupEntries::sequential((&texture.texture_view, &texture.sampler)),
                ),
            );
        }
    }
}
