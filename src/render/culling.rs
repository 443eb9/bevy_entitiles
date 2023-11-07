use bevy::prelude::{Component, Query, ResMut};

use crate::{
    math::{
        aabb::AabbBox2d,
        extension::Vec2ToUVec2,
        tilemap::world_pos_to_chunk_square,
    },
    tilemap::TileType,
};

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

            let camera_aabb = AabbBox2d::from_camera(camera).with_uniform_scale(0.2);

            match tilemap.tile_type {
                TileType::Square => cull_square(&camera_aabb, tilemap, &mut render_chunk_storage),
                TileType::IsometricDiamond => {
                    cull_isometric_diamond(&camera_aabb, tilemap, &mut render_chunk_storage)
                }
            }
        }
    }
}

fn cull_square(
    camera_aabb: &AabbBox2d,
    tilemap: &ExtractedTilemap,
    render_chunk_storage: &mut ResMut<RenderChunkStorage>,
) {
    let Some(storage_size) = render_chunk_storage.get_storage_size(tilemap.id) else {
        return;
    };
    let chunks = render_chunk_storage.get_chunks_mut(tilemap.id).unwrap();

    let min_chunk_index = world_pos_to_chunk_square(
        tilemap.transfrom,
        tilemap.render_chunk_size,
        tilemap.tile_render_size,
        camera_aabb.min,
    )
    .floor_to_uvec();
    let max_chunk_index = world_pos_to_chunk_square(
        tilemap.transfrom,
        tilemap.render_chunk_size,
        tilemap.tile_render_size,
        camera_aabb.max,
    )
    .ceil_to_uvec();

    for index_y in min_chunk_index.y..max_chunk_index.y {
        for index_x in min_chunk_index.x..max_chunk_index.x {
            if let Some(Some(chunk)) = chunks.get_mut((index_y * storage_size.x + index_x) as usize)
            {
                chunk.visible = true;
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
            if c.aabb.is_intersected_with(camera_aabb) {
                c.visible = true;
            }
        }
    }
}
