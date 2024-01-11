use bevy::{
    ecs::system::{Res, Resource},
    prelude::{Query, ResMut},
};

use super::{
    chunk::RenderChunkStorage,
    extract::{ExtractedTilemap, ExtractedView},
};

#[derive(Resource)]
pub struct FrustumCulling(pub bool);

impl Default for FrustumCulling {
    fn default() -> Self {
        Self(true)
    }
}

pub fn cull_chunks(
    tilemaps: Query<&ExtractedTilemap>,
    mut render_chunk_storage: ResMut<RenderChunkStorage>,
    cameras: Query<&ExtractedView>,
    culling: Res<FrustumCulling>,
) {
    if !culling.0 {
        return;
    }

    cameras.for_each(|cam_aabb| {
        tilemaps.for_each(|tilemap| {
            let Some(chunks) = render_chunk_storage.get_chunks_mut(tilemap.id) else {
                return;
            };

            chunks.values_mut().for_each(|c| {
                if c.aabb.is_intersected(cam_aabb.0) {
                    c.visible = true;
                } else {
                    c.visible = false;
                }
            });
        });
    });
}
