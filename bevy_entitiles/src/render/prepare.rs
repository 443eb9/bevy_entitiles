use bevy::{
    prelude::{Commands, Query, Res, ResMut},
    render::renderer::{RenderDevice, RenderQueue},
};

use super::{
    buffer::{TilemapUniformBuffers, UniformBuffer},
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
    // mut storage_buffers: ResMut<TilemapStorageBuffers>,
    mut textures_storage: ResMut<TilemapTexturesStorage>,
) {
    render_chunks.add_tiles_with_query(&extracted_tilemaps, &extracted_tiles);

    uniform_buffers.clear();
    // storage_buffers.clear();

    for tilemap in extracted_tilemaps.iter() {
        commands
            .entity(tilemap.id)
            .insert(uniform_buffers.insert(tilemap));

        render_chunks.prepare_chunks(tilemap, &render_device);

        if let Some(texture) = tilemap.texture.as_ref() {
            if textures_storage.contains(&texture.texture) {
                continue;
            }

            textures_storage.insert(texture.clone_weak(), texture.desc());
            // storage_buffers.insert_anim_seqs(tilemap.id, &tilemap.anim_seqs.to_vec());
        }
    }

    textures_storage.prepare_textures(&render_device);
    uniform_buffers.write(&render_device, &render_queue);
    // storage_buffers.write(&render_device, &render_queue);
}
