use bevy::prelude::{Component, Query, ResMut};

use crate::{math::aabb::AabbBox2d, tilemap::tile::TileType};

use super::{
    chunk::RenderChunkStorage,
    extract::{ExtractedTilemap, ExtractedView},
};

#[derive(Component)]
pub struct VisibleTilemap;

pub fn cull(
    visible_tilemaps: Query<&ExtractedTilemap>,
    mut render_chunk_storage: ResMut<RenderChunkStorage>,
    cameras: Query<&ExtractedView>,
) {
    for tilemap in visible_tilemaps.iter() {
        for camera in cameras.iter() {
            let Some(chunks) = render_chunk_storage.get_chunks_mut(tilemap.id) else {
                break;
            };

            for chunk in chunks.iter_mut() {
                if let Some(c) = chunk {
                    c.visible = false;
                }
            }

            let camera_aabb = AabbBox2d::from_camera(camera);
            let chunks = render_chunk_storage.get_chunks_mut(tilemap.id).unwrap();

            // TODO Optimize
            for chunk in chunks.iter_mut() {
                if let Some(c) = chunk {
                    if c.aabb.is_intersected(&camera_aabb) {
                        c.visible = true;
                    }
                }
            }
        }
    }
}
