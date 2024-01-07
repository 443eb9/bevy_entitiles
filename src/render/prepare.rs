use bevy::{
    ecs::entity::Entity,
    prelude::{Commands, Query, Res, ResMut},
    render::renderer::{RenderDevice, RenderQueue},
};

use super::{
    buffer::{TilemapUniformBuffers, UniformBuffer},
    chunk::{TilemapRenderChunk, UnloadedRenderChunk},
    extract::{ExtractedTile, ExtractedTilemap},
    texture::TilemapTexturesStorage,
    RenderChunkStorage,
};

pub fn prepare_tilemaps(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    extracted_tilemaps: Query<&ExtractedTilemap>,
    mut render_chunks: ResMut<RenderChunkStorage>,
    mut uniform_buffers: ResMut<TilemapUniformBuffers>,
    // mut storage_buffers: ResMut<TilemapStorageBuffers>,
    mut textures_storage: ResMut<TilemapTexturesStorage>,
) {
    uniform_buffers.clear();
    // storage_buffers.clear();

    extracted_tilemaps.for_each(|tilemap| {
        commands
            .entity(tilemap.id)
            .insert(uniform_buffers.insert(tilemap));

        render_chunks.prepare_chunks(tilemap, &render_device);

        if let Some(texture) = tilemap.texture.as_ref() {
            if textures_storage.contains(&texture.texture) {
                return;
            }

            textures_storage.insert(texture.clone_weak(), texture.desc());
            // storage_buffers.insert_anim_seqs(tilemap.id, &tilemap.anim_seqs.to_vec());
        }
    });

    textures_storage.prepare_textures(&render_device);
    uniform_buffers.write(&render_device, &render_queue);
    // storage_buffers.write(&render_device, &render_queue);
}

pub fn prepare_tiles(
    extracted_tilemaps: Query<&ExtractedTilemap>,
    extracted_tiles: Query<&mut ExtractedTile>,
    mut render_chunks: ResMut<RenderChunkStorage>,
) {
    extracted_tiles.for_each(|tile| {
        let Ok(tilemap) = extracted_tilemaps.get(tile.tilemap) else {
            return;
        };

        let chunks = render_chunks.value.entry(tile.tilemap).or_default();

        let chunk = chunks
            .entry(tile.chunk_index)
            .or_insert_with(|| TilemapRenderChunk::from_index(tile.chunk_index, tilemap));

        chunk.set_tile(tile.in_chunk_index, tile);
    });
}

pub fn prepare_unloaded_chunks(
    mut render_chunks: ResMut<RenderChunkStorage>,
    extracted_tilemaps: Query<(Entity, &UnloadedRenderChunk)>,
) {
    extracted_tilemaps.for_each(|(entity, unloaded)| {
        unloaded.0.iter().for_each(|c| {
            render_chunks.remove_chunk(entity, *c);
        });
    });
}
