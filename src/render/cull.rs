use bevy::{
    ecs::system::{Res, Resource},
    prelude::{Query, ResMut},
    render::view::ViewVisibility,
};

use crate::{
    math::CameraAabb2d,
    render::{
        chunk::RenderChunkStorage,
        extract::{ExtractedView, TilemapInstances},
    },
    tilemap::map::TilemapAabbs,
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
            if !aabbs.world_aabb.intersect(camera.0).is_empty() {
                visibility.set();
            }
        });
    });
}

pub fn cull_chunks(
    tilemaps: Res<TilemapInstances>,
    mut render_chunk_storage: ResMut<RenderChunkStorage>,
    cameras: Query<&ExtractedView>,
    culling: Res<FrustumCulling>,
) {
    if !culling.0 {
        return;
    }

    // tilemaps.keys().for_each(|tilemap| {
    //     render_chunk_storage
    //         .get_or_insert_chunks(*tilemap)
    //         .value
    //         .values_mut()
    //         .for_each(|c| {
    //             c.visible = false;
    //         });
    // });

    // cameras.iter().for_each(|cam_aabb| {
    //     tilemaps.keys().for_each(|tilemap| {
    //         render_chunk_storage
    //             .get_or_insert_chunks(*tilemap)
    //             .value
    //             .values_mut()
    //             .for_each(|c| {
    //                 if !c.aabb.intersect(cam_aabb.0).is_empty() {
    //                     c.visible = true;
    //                 }
    //             });
    //     });
    // });
}
