use bevy::{
    prelude::{Commands, Query, Res, ResMut},
    render::{
        render_asset::RenderAssets,
        render_resource::{AddressMode, SamplerDescriptor},
        renderer::{RenderDevice, RenderQueue},
        texture::Image,
    },
    time::Time,
};

use super::{
    extract::{ExtractedTile, ExtractedTilemap},
    texture::TilemapTexturesStorage,
    uniform::{TilemapUniformsStorage, UniformsStorage},
    RenderChunkStorage,
};

pub fn prepare(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    extracted_tilemaps: Query<&ExtractedTilemap>,
    extracted_tiles: Query<&mut ExtractedTile>,
    mut render_chunks: ResMut<RenderChunkStorage>,
    mut tilemap_uniforms_storage: ResMut<TilemapUniformsStorage>,
    mut textures_storage: ResMut<TilemapTexturesStorage>,
    render_images: Res<RenderAssets<Image>>,
    time: Res<Time>,
) {
    render_chunks.add_tiles_with_query(&extracted_tilemaps, &extracted_tiles);

    tilemap_uniforms_storage.clear();
    for tilemap in extracted_tilemaps.iter() {
        commands
            .entity(tilemap.id)
            .insert(tilemap_uniforms_storage.insert(tilemap));

        render_chunks.prepare_chunks(tilemap, &render_device, &time);

        if let Some(tex) = tilemap.texture.as_ref() {
            if textures_storage.contains(&tex.texture) {
                continue;
            }

            if let Some(rd_img) = render_images.get(tex.handle()) {
                textures_storage.insert(
                    tex.clone_weak(),
                    rd_img.clone(),
                    render_device.create_sampler(&SamplerDescriptor {
                        label: Some("tilemap_color_texture_sampler"),
                        address_mode_u: AddressMode::ClampToEdge,
                        address_mode_v: AddressMode::ClampToEdge,
                        address_mode_w: AddressMode::ClampToEdge,
                        mag_filter: tex.desc.filter_mode,
                        min_filter: tex.desc.filter_mode,
                        mipmap_filter: tex.desc.filter_mode,
                        lod_min_clamp: 0.,
                        lod_max_clamp: f32::MAX,
                        compare: None,
                        anisotropy_clamp: 1,
                        border_color: None,
                    }),
                );
            }
        }
    }
    tilemap_uniforms_storage.write(&render_device, &render_queue);
}
