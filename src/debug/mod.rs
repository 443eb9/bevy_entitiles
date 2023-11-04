use bevy::{
    math::Vec3Swizzles,
    prelude::{Color, Gizmos, Plugin, Query, Transform, UVec2, Update, Vec2},
};

use crate::{
    math::geometry::AabbBox2d,
    render::{chunk::RenderChunkStorage, extract::ExtractedTilemap},
    tilemap::Tilemap,
};

/// A bunch of systems for debugging. Since they're not optimized, don't use them unless you're debugging.
pub struct EntiTilesDebugPlugin;

impl Plugin for EntiTilesDebugPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Update, (draw_tilemap_aabb, draw_chunk_aabb));
    }
}

pub fn draw_tilemap_aabb(mut gizmos: Gizmos, tilemaps: Query<(&Tilemap, &Transform)>) {
    for (tilemap, transform) in tilemaps.iter() {
        gizmos.rect_2d(
            tilemap.aabb.center + transform.translation.xy(),
            0.,
            Vec2::new(tilemap.aabb.width, tilemap.aabb.height),
            Color::RED,
        )
    }
}

pub fn draw_chunk_aabb(mut gizmos: Gizmos, tilemaps: Query<(&Tilemap, &Transform)>) {
    for (tilemap, tilemap_transform) in tilemaps.iter() {
        let tilemap = ExtractedTilemap {
            id: tilemap.id,
            tile_type: tilemap.tile_type.clone(),
            size: tilemap.size,
            tile_size: tilemap.tile_size,
            tile_render_size: tilemap.tile_render_size,
            render_chunk_size: tilemap.render_chunk_size,
            filter_mode: tilemap.filter_mode,
            texture: tilemap.texture.clone(),
            transfrom: *tilemap_transform,
            transform_matrix: tilemap_transform.compute_matrix(),
            flip: tilemap.flip,
            aabb: tilemap.aabb.clone(),
        };
        let count = RenderChunkStorage::calculate_render_chunk_count(&tilemap);

        for y in 0..count.y {
            for x in 0..count.x {
                let aabb = AabbBox2d::from_chunk(UVec2::new(x, y), &tilemap);
                gizmos.rect_2d(
                    aabb.center + tilemap_transform.translation.xy(),
                    0.,
                    Vec2::new(aabb.width, aabb.height),
                    Color::GREEN,
                );
            }
        }
    }
}
