use bevy::{
    asset::{AssetId, Handle},
    ecs::entity::EntityHashMap,
    prelude::{Res, ResMut, Resource},
    render::{
        render_asset::RenderAssets,
        render_resource::{BindGroup, BindGroupEntries},
        renderer::RenderDevice,
        texture::{FallbackImage, GpuImage},
        view::ViewUniforms,
    },
    utils::HashMap,
};

use crate::{
    render::{
        buffer::TilemapBuffers,
        extract::{TilemapInstances, TilemapMaterialIds},
        material::{ExtractedTilemapMaterialWrapper, TilemapMaterial},
        pipeline::EntiTilesPipeline,
        texture::TilemapTexturesStorage,
    },
    tilemap::map::TilemapTextures,
};

#[derive(Resource, Default)]
pub struct TilemapBindGroups<M: TilemapMaterial> {
    pub uniform_buffer: Option<BindGroup>,
    pub array_buffers: EntityHashMap<BindGroup>,
    pub textures: HashMap<Handle<TilemapTextures>, BindGroup>,
    pub materials: HashMap<AssetId<M>, BindGroup>,
}

pub fn bind_tilemap_buffers<M: TilemapMaterial>(
    render_device: Res<RenderDevice>,
    entitiles_pipeline: Res<EntiTilesPipeline<M>>,
    tilemap_buffers: Res<TilemapBuffers>,
    mut bind_groups: ResMut<TilemapBindGroups<M>>,
    tilemap_instances: Res<TilemapInstances>,
    view_uniforms: Res<ViewUniforms>,
) {
    if let (Some(uniform_buffer), Some(view_binding)) = (
        tilemap_buffers.shared.uniform.binding(),
        view_uniforms.uniforms.binding(),
    ) {
        bind_groups.uniform_buffer = Some(render_device.create_bind_group(
            Some("tilemap_uniform_buffers_bind_group"),
            &entitiles_pipeline.uniform_buffers_layout,
            &BindGroupEntries::sequential((uniform_buffer, view_binding)),
        ));
    }

    for tilemap in tilemap_instances.keys() {
        let Some(buffers) = tilemap_buffers.unshared.get(tilemap) else {
            continue;
        };

        let Some(anim) = buffers.animation.binding() else {
            continue;
        };

        #[cfg(feature = "atlas")]
        let Some(desc) = buffers.texture_desc.binding() else {
            continue;
        };

        bind_groups.array_buffers.insert(
            *tilemap,
            render_device.create_bind_group(
                "tilemap_storage_buffers_bind_group",
                &entitiles_pipeline.array_buffers_layout,
                &BindGroupEntries::sequential((
                    anim,
                    #[cfg(feature = "atlas")]
                    desc,
                )),
            ),
        );
    }
}

pub fn bind_materials<M: TilemapMaterial>(
    pipeline: Res<EntiTilesPipeline<M>>,
    render_device: Res<RenderDevice>,
    images: Res<RenderAssets<GpuImage>>,
    fallback_image: Res<FallbackImage>,
    material_ids: Res<TilemapMaterialIds<M>>,
    mut bind_groups: ResMut<TilemapBindGroups<M>>,
    material_assets: Res<RenderAssets<ExtractedTilemapMaterialWrapper<M>>>,
) {
    for id in material_ids.values() {
        if let Some(material) = material_assets.get(*id) {
            let bind_group = material
                .as_bind_group(
                    &pipeline.material_layout,
                    &render_device,
                    &images,
                    &fallback_image,
                )
                .unwrap();
            bind_groups.materials.insert(*id, bind_group.bind_group);
        }
    }
}

pub fn bind_textures<M: TilemapMaterial>(
    tilemap_instances: Res<TilemapInstances>,
    render_device: Res<RenderDevice>,
    textures_storage: Res<TilemapTexturesStorage>,
    entitiles_pipeline: Res<EntiTilesPipeline<M>>,
    mut bind_groups: ResMut<TilemapBindGroups<M>>,
) {
    for tilemap in tilemap_instances.values() {
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
