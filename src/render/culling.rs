use bevy::prelude::{Component, Query, ResMut};

use crate::math::aabb::AabbBox2d;

use super::{
    chunk::RenderChunkStorage,
    extract::{ExtractedTilemap, ExtractedView},
};

#[derive(Component)]
pub struct Visible;

pub fn cull(
    visible_tilemaps: Query<&ExtractedTilemap>,
    mut render_chunk_storage: ResMut<RenderChunkStorage>,
    cameras: Query<&ExtractedView>,
) {
    for camera in cameras.iter() {
        let camera_aabb = AabbBox2d::from_camera(camera);

        for tilemap in visible_tilemaps.iter() {
            frustum_culling(&camera_aabb, &mut render_chunk_storage);
        }
    }
}

fn frustum_culling(camera_aabb: &AabbBox2d, render_chunk_storage: &mut ResMut<RenderChunkStorage>) {
}
