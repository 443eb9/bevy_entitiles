use bevy::{
    color::palettes::css::{BLUE, GREEN, RED},
    ecs::system::Query,
    gizmos::gizmos::Gizmos,
    math::{Rect, Vec2},
};

use crate::{
    math::{ext::RectFromTilemap, CameraAabb2d},
    tilemap::map::{
        TilePivot, TilemapAabbs, TilemapAxisFlip, TilemapSlotSize, TilemapStorage,
        TilemapTransform, TilemapType,
    },
};

#[cfg(feature = "algorithm")]
use crate::algorithm::pathfinding::Path;

#[cfg(feature = "serializing")]
use crate::math::ext::RectTransformation;

pub fn draw_chunk_aabb(
    mut gizmos: Gizmos,
    tilemaps: Query<(
        &TilemapType,
        &TilePivot,
        &TilemapAxisFlip,
        &TilemapSlotSize,
        &TilemapTransform,
        &TilemapStorage,
    )>,
) {
    for (ty, tile_pivot, axis_flip, slot_size, transform, storage) in tilemaps.iter() {
        storage.storage.chunks.keys().for_each(|chunk| {
            let aabb = Rect::from_tilemap(
                *chunk,
                storage.storage.chunk_size,
                *ty,
                tile_pivot.0,
                *axis_flip,
                slot_size.0,
                *transform,
            );
            gizmos.rect_2d(
                aabb.center(),
                0.,
                Vec2::new(aabb.width(), aabb.height()),
                GREEN,
            );
        });
    }
}

pub fn draw_tilemap_aabb(mut gizmos: Gizmos, tilemaps: Query<&TilemapAabbs>) {
    tilemaps.iter().for_each(|aabb| {
        gizmos.rect_2d(
            aabb.world_aabb.center(),
            0.,
            Vec2::new(aabb.world_aabb.width(), aabb.world_aabb.height()),
            RED,
        );
    });
}

#[cfg(feature = "algorithm")]
pub fn draw_path(
    mut gizmos: Gizmos,
    path_query: Query<&Path>,
    tilemaps: Query<(
        &TilemapType,
        &TilemapTransform,
        &TilePivot,
        &TilemapSlotSize,
    )>,
) {
    for path in path_query.iter() {
        let (ty, transform, pivot, slot_size) = tilemaps.get(path.tilemap()).unwrap();

        for node in path.iter() {
            gizmos.circle_2d(
                crate::tilemap::coordinates::index_to_world(
                    *node,
                    *ty,
                    transform,
                    pivot.0,
                    slot_size.0,
                ),
                10.,
                bevy::color::palettes::css::GREEN_YELLOW,
            );
        }
    }
}

pub fn draw_axis(mut gizmos: Gizmos) {
    gizmos.line_2d(Vec2::NEG_X * 1e10, Vec2::X * 1e10, RED);
    gizmos.line_2d(Vec2::NEG_Y * 1e10, Vec2::Y * 1e10, GREEN);
}

pub fn draw_camera_aabb(mut gizmos: Gizmos, camera_aabb: Query<&CameraAabb2d>) {
    camera_aabb.iter().for_each(|aabb| {
        gizmos.rect_2d(
            aabb.center(),
            0.,
            Vec2::new(aabb.width(), aabb.height()),
            BLUE,
        );
    });
}

#[cfg(feature = "serializing")]
pub fn draw_updater_aabbs(
    mut gizmos: Gizmos,
    cameras_query: Query<(
        &CameraAabb2d,
        &crate::tilemap::chunking::camera::CameraChunkUpdater,
    )>,
) {
    cameras_query.iter().for_each(|(cam_aabb, cam_updater)| {
        let detect_aabb =
            cam_aabb.with_scale(Vec2::splat(cam_updater.detect_scale), Vec2::splat(0.5));
        let update_aabb =
            cam_aabb.with_scale(Vec2::splat(cam_updater.update_scale), Vec2::splat(0.5));

        gizmos.rect_2d(
            detect_aabb.center(),
            0.,
            Vec2::new(detect_aabb.width(), detect_aabb.height()),
            bevy::color::palettes::css::FUCHSIA,
        );
        gizmos.rect_2d(
            update_aabb.center(),
            0.,
            Vec2::new(update_aabb.width(), update_aabb.height()),
            bevy::color::palettes::css::SILVER,
        );
    });
}
