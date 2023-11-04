use bevy::{
    math::Vec3Swizzles,
    prelude::{Commands, Component, Query, ResMut, Transform, With},
};

use crate::math::geometry::AabbBox2d;

use super::{
    chunk::RenderChunkStorage,
    extract::{ExtractCamera, ExtractedTilemap},
};

#[derive(Component)]
pub struct Visible;

pub fn cull_tilemaps(
    mut commands: Commands,
    mut tilemaps: Query<(&mut ExtractedTilemap, &Transform)>,
    cameras: Query<&ExtractCamera>,
) {
    for camera in cameras.iter() {
        let camera_aabb = AabbBox2d::from_camera(camera);

        for (tilemap, transform) in tilemaps.iter_mut() {
            if tilemap.aabb.is_intersected_with(
                transform.translation.xy(),
                &camera_aabb,
                camera.transform,
            ) {
                commands.entity(tilemap.id).insert(Visible);
            } else {
                commands.entity(tilemap.id).remove::<Visible>();
            }
        }
    }
}

pub fn cull_chunks(
    visible_tilemaps: Query<&ExtractedTilemap>,
    mut render_chunk_storage: ResMut<RenderChunkStorage>,
    cameras: Query<&ExtractCamera>,
) {
    for camera in cameras.iter() {
        let mut camera_aabb = AabbBox2d::from_camera(camera);

        for tilemap in visible_tilemaps.iter() {
            let tilemap_position = tilemap.transfrom.translation.xy();

            if let Some(chunks) = render_chunk_storage.get_mut(tilemap.id) {
                for chunk in chunks.iter_mut() {
                    if let Some(chunk) = chunk {
                        if chunk.aabb.is_intersected_with(
                            tilemap_position,
                            &camera_aabb,
                            camera.transform,
                        ) {
                            chunk.visible = true;
                        } else {
                            chunk.visible = false;
                        }
                    }
                }
            }
        }
    }
}
