use bevy::{
    prelude::{Res, ResMut},
    render::renderer::RenderDevice,
};

use super::{RenderChunkStorage, texture::TilemapTextureArrayStorage};

pub fn prepare(
    render_device: Res<RenderDevice>,
    mut render_chunks: ResMut<RenderChunkStorage>,
    mut tilemap_texture_array_storage: ResMut<TilemapTextureArrayStorage>,
) {
    for (_, chunks) in render_chunks.value.iter_mut() {
        for c in chunks {
            c.update_mesh(&render_device);
        }
    }

    tilemap_texture_array_storage.prepare(&render_device);
}
