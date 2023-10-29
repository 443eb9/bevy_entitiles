use bevy::{
    prelude::{Res, ResMut},
    render::renderer::RenderDevice,
};

use super::RenderChunkStorage;

pub fn prepare(
    render_device: Res<RenderDevice>,
    mut render_chunks: ResMut<RenderChunkStorage>,
) {
    for (_, chunks) in render_chunks.value.iter_mut() {
        for c in chunks {
            c.update_mesh(&render_device);
        }
    }
}
