use bevy::{
    prelude::{Commands, Entity, Query, Res, ResMut},
    render::renderer::{RenderDevice, RenderQueue},
};

use super::{
    extract::{ExtractedTile, ExtractedTilemap},
    texture::TilemapTextureArrayStorage,
    uniform::TilemapUniformsStorage,
    RenderChunkStorage,
};

pub fn prepare(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    extracted_tilemaps: Query<&ExtractedTilemap>,
    extracted_tiles: Query<&ExtractedTile>,
    mut render_chunks: ResMut<RenderChunkStorage>,
    mut tilemap_texture_array_storage: ResMut<TilemapTextureArrayStorage>,
    mut tilemap_uniforms_storage: ResMut<TilemapUniformsStorage>,
) {
    render_chunks.add_tiles_with_query(&extracted_tilemaps, &extracted_tiles);

    tilemap_uniforms_storage.reset();
    for tilemap in extracted_tilemaps.iter() {
        commands
            .entity(tilemap.id)
            .insert(tilemap_uniforms_storage.insert(tilemap));

        render_chunks.prepare_chunks(tilemap, &render_device);
    }
    tilemap_uniforms_storage.write(&render_device, &render_queue);

    tilemap_texture_array_storage.prepare(&render_device);
}
