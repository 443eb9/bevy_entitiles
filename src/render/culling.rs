use bevy::prelude::{Component, Query, ResMut};

use crate::{
    math::{aabb::AabbBox2d, extension::Vec2ToUVec2, tilemap::world_pos_to_chunk_square},
    tilemap::tile::TileType,
};

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

            match tilemap.tile_type {
                TileType::Square => cull_square(&camera_aabb, tilemap, &mut render_chunk_storage),
                TileType::IsometricDiamond => {
                    cull_isometric_diamond(&camera_aabb, tilemap, &mut render_chunk_storage)
                }
            }
        }
    }
}

// TODO optimize
fn cull_square(
    camera_aabb: &AabbBox2d,
    tilemap: &ExtractedTilemap,
    render_chunk_storage: &mut ResMut<RenderChunkStorage>,
) {
    let chunks = render_chunk_storage.get_chunks_mut(tilemap.id).unwrap();

    for chunk in chunks.iter_mut() {
        if let Some(c) = chunk {
            if c.aabb.is_intersected(camera_aabb) {
                c.visible = true;
            }
        }
    }
}

fn cull_isometric_diamond(
    camera_aabb: &AabbBox2d,
    tilemap: &ExtractedTilemap,
    render_chunk_storage: &mut ResMut<RenderChunkStorage>,
) {
    let chunks = render_chunk_storage.get_chunks_mut(tilemap.id).unwrap();

    for chunk in chunks.iter_mut() {
        if let Some(c) = chunk {
            if c.aabb.is_intersected(camera_aabb) {
                c.visible = true;
            }
        }
    }
}
