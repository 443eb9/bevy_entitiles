use bevy::{
    prelude::{Commands, Query, Res, ResMut, Without},
    render::renderer::{RenderDevice, RenderQueue},
    time::Time,
};

use super::{
    culling::VisibleTilemap,
    extract::{ExtractedTile, ExtractedTilemap},
    texture::TilemapTextureArrayStorage,
    uniform::TilemapUniformsStorage,
    RenderChunkStorage,
};

pub fn prepare(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    extracted_tilemaps: Query<&ExtractedTilemap, Without<VisibleTilemap>>,
    extracted_tiles: Query<&mut ExtractedTile>,
    mut render_chunks: ResMut<RenderChunkStorage>,
    mut tilemap_texture_array_storage: ResMut<TilemapTextureArrayStorage>,
    mut tilemap_uniforms_storage: ResMut<TilemapUniformsStorage>,
    time: Res<Time>,
) {
    render_chunks.add_tiles_with_query(&extracted_tilemaps, &extracted_tiles);

    tilemap_uniforms_storage.clear();
    for tilemap in extracted_tilemaps.iter() {
        commands
            .entity(tilemap.id)
            .insert(tilemap_uniforms_storage.insert(tilemap));

        render_chunks.prepare_chunks(tilemap, &render_device, &time);
    }
    tilemap_uniforms_storage.write(&render_device, &render_queue);

    tilemap_texture_array_storage.prepare_textures(&render_device);
}
