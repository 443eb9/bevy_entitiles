use bevy::{
    prelude::{Res, ResMut},
    render::{
        renderer::{RenderDevice, RenderQueue},
    },
};

use super::{ExtractedData, TileData, UniformData};

pub fn prepare(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut uniform_data: ResMut<UniformData>,
    extracted_tiles: Res<ExtractedData>,
) {
    for t in extracted_tiles.tiles.iter() {
        uniform_data.tile_data.push(TileData {
            texture_index: t.texture_index,
            coordinate: t.coordinate,
        });
    }

    uniform_data
        .tile_data
        .write_buffer(&render_device, &render_queue);
}
