use bevy::{
    prelude::{Commands, Query, Res, ResMut},
    render::{
        render_asset::RenderAssets,
        render_resource::{AddressMode, SamplerDescriptor},
        renderer::{RenderDevice, RenderQueue},
        texture::Image,
    },
};

use super::{
    buffer::{TilemapStorageBuffers, TilemapUniformBuffers, UniformBuffer},
    extract::{ExtractedTile, ExtractedTilemap},
    texture::TilemapTexturesStorage,
    RenderChunkStorage,
};

pub fn prepare(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    extracted_tilemaps: Query<&ExtractedTilemap>,
    extracted_tiles: Query<&mut ExtractedTile>,
    mut render_chunks: ResMut<RenderChunkStorage>,
    mut uniform_buffers: ResMut<TilemapUniformBuffers>,
    mut storage_buffers: ResMut<TilemapStorageBuffers>,
    mut textures_storage: ResMut<TilemapTexturesStorage>,
    render_images: Res<RenderAssets<Image>>,
) {
    render_chunks.add_tiles_with_query(&extracted_tilemaps, &extracted_tiles);

    uniform_buffers.clear();
    storage_buffers.clear();

    for tilemap in extracted_tilemaps.iter() {
        commands
            .entity(tilemap.id)
            .insert(uniform_buffers.insert(tilemap));

        render_chunks.prepare_chunks(tilemap, &render_device);

        if let Some(texture) = tilemap.texture.as_ref() {
            if textures_storage.contains(&texture.texture) {
                continue;
            }

            if let Some(rd_img) = render_images.get(texture.handle()) {
                textures_storage.insert(
                    texture.clone_weak(),
                    rd_img.clone(),
                    render_device.create_sampler(&SamplerDescriptor {
                        label: Some("tilemap_color_texture_sampler"),
                        address_mode_u: AddressMode::ClampToEdge,
                        address_mode_v: AddressMode::ClampToEdge,
                        address_mode_w: AddressMode::ClampToEdge,
                        mag_filter: texture.desc.filter_mode,
                        min_filter: texture.desc.filter_mode,
                        mipmap_filter: texture.desc.filter_mode,
                        lod_min_clamp: 0.,
                        lod_max_clamp: f32::MAX,
                        compare: None,
                        anisotropy_clamp: 1,
                        border_color: None,
                    }),
                );
            }

            storage_buffers.insert_atlas_uvs(tilemap.id, &texture.desc().tiles_uv);
            storage_buffers.insert_anim_seqs(tilemap.id, &tilemap.anim_seqs.to_vec());
        }
    }

    uniform_buffers.write(&render_device, &render_queue);
    storage_buffers.write(&render_device, &render_queue);
}
