use bevy::{
    math::Vec3Swizzles,
    prelude::{Commands, Component, Query, ResMut, Transform, With},
    render::camera,
};

use crate::math::aabb::AabbBox2d;

use super::{
    chunk::RenderChunkStorage,
    extract::{ExtractedTilemap, ExtractedView},
};

#[derive(Component)]
pub struct Visible;

pub fn cull_tilemaps(
    mut commands: Commands,
    mut tilemaps: Query<(&mut ExtractedTilemap, &Transform)>,
    cameras: Query<&ExtractedView>,
) {
    for camera in cameras.iter() {
        let camera_aabb = AabbBox2d::from_camera(camera);

        for (mut tilemap, transform) in tilemaps.iter_mut() {
            if tilemap.aabb.is_intersected_with(&camera_aabb) {
                commands.entity(tilemap.id).insert(Visible);
                println!("visible");
            } else {
                commands.entity(tilemap.id).remove::<Visible>();
            }
        }
    }
}

pub fn cull_chunks(
    visible_tilemaps: Query<&ExtractedTilemap>,
    mut render_chunk_storage: ResMut<RenderChunkStorage>,
    cameras: Query<&ExtractedView>,
) {
    for camera in cameras.iter() {
        let camera_aabb = AabbBox2d::from_camera(camera);

        for tilemap in visible_tilemaps.iter() {
            let tilemap_position = tilemap.transfrom.translation.xy();

            if let Some(chunks) = render_chunk_storage.get_mut(tilemap.id) {
                for chunk in chunks.iter_mut() {
                    if let Some(chunk) = chunk {
                        if chunk.aabb.is_intersected_with(&camera_aabb) {
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
