use bevy::prelude::{Query, ResMut};

use crate::math::aabb::Aabb2d;

use super::{
    chunk::RenderChunkStorage,
    extract::{ExtractedTilemap, ExtractedView},
};

pub fn cull_chunks(
    tilemaps: Query<&ExtractedTilemap>,
    mut render_chunk_storage: ResMut<RenderChunkStorage>,
    cameras: Query<&ExtractedView>,
) {
    let Ok(proj) = cameras.get_single() else {
        return;
    };

    for tilemap in tilemaps.iter() {
        let Some(chunks) = render_chunk_storage.get_chunks_mut(tilemap.id) else {
            break;
        };

        let camera_aabb = Aabb2d::from_camera(proj);

        chunks.values_mut().for_each(|c| {
            if c.aabb.is_intersected(&camera_aabb) {
                c.visible = true;
            } else {
                c.visible = false;
            }
        })
    }
}
