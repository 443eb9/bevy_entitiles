use bevy::{
    ecs::system::{Res, Resource},
    prelude::{Query, ResMut},
    render::view::ViewVisibility,
};

use crate::{math::CameraAabb2d, tilemap::map::TilemapAabbs};

use super::{
    chunk::RenderChunkStorage,
    extract::{ExtractedTilemap, ExtractedView},
    material::TilemapMaterial,
};

#[derive(Resource)]
pub struct FrustumCulling(pub bool);

impl Default for FrustumCulling {
    fn default() -> Self {
        Self(true)
    }
}

pub fn cull_tilemaps(
    mut tilemaps: Query<(&TilemapAabbs, &mut ViewVisibility)>,
    cameras: Query<&CameraAabb2d>,
    culling: Res<FrustumCulling>,
) {
    if !culling.0 {
        return;
    }

    cameras.iter().for_each(|camera| {
        tilemaps.par_iter_mut().for_each(|(aabbs, mut visibility)| {
            if aabbs.world_aabb.is_intersected(camera.0) {
                visibility.set();
            }
        });
    });
}

pub fn cull_chunks<M: TilemapMaterial>(
    tilemaps: Query<&ExtractedTilemap<M>>,
    mut render_chunk_storage: ResMut<RenderChunkStorage<M>>,
    cameras: Query<&ExtractedView>,
    culling: Res<FrustumCulling>,
) {
    if !culling.0 {
        return;
    }

    tilemaps.iter().for_each(|tilemap| {
        render_chunk_storage
            .get_or_insert_chunks(tilemap.id)
            .value
            .values_mut()
            .for_each(|c| {
                c.visible = false;
            });
    });

    cameras.iter().for_each(|cam_aabb| {
        tilemaps.iter().for_each(|tilemap| {
            render_chunk_storage
                .get_or_insert_chunks(tilemap.id)
                .value
                .values_mut()
                .for_each(|c| {
                    if c.aabb.is_intersected(cam_aabb.0) {
                        c.visible = true;
                    }
                });
        });
    });
}
