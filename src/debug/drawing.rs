use bevy::{ecs::system::Query, gizmos::gizmos::Gizmos, math::Vec2, render::color::Color};

use crate::{
    math::aabb::Aabb2d,
    tilemap::{map::Tilemap, tile::Tile},
};

#[cfg(feature = "algorithm")]
use crate::algorithm::pathfinding::Path;

// #[cfg(feature = "debug")]
// pub fn draw_tilemap_aabb(mut gizmos: Gizmos, tilemaps: Query<&Tilemap>) {
//     for tilemap in tilemaps.iter() {
//         let tilemap = PubTilemap::from_tilemap(tilemap);
//         gizmos.rect_2d(
//             tilemap.aabb.center(),
//             0.,
//             Vec2::new(tilemap.aabb.width(), tilemap.aabb.height()),
//             Color::RED,
//         );
//         gizmos.circle_2d(tilemap.aabb.min, 5., Color::ORANGE);
//         gizmos.circle_2d(tilemap.aabb.max, 5., Color::CYAN);
//     }
// }

#[cfg(feature = "debug")]
pub fn draw_chunk_aabb(mut gizmos: Gizmos, tilemaps: Query<&Tilemap>) {
    use crate::render::extract::ExtractedTilemap;

    for tilemap in tilemaps.iter() {
        tilemap.storage.chunks.keys().for_each(|chunk| {
            let aabb = Aabb2d::from_chunk(
                *chunk,
                &ExtractedTilemap {
                    id: tilemap.id,
                    tile_type: tilemap.tile_type,
                    ext_dir: tilemap.ext_dir,
                    tile_render_size: tilemap.tile_render_size,
                    tile_slot_size: tilemap.tile_slot_size,
                    chunk_size: tilemap.chunk_size,
                    pivot: tilemap.pivot,
                    texture: tilemap.texture.clone(),
                    transform: tilemap.transform,
                    anim_seqs: tilemap.anim_seqs,
                    layer_opacities: tilemap.layer_opacities,
                    time: 0.,
                },
            );
            gizmos.rect_2d(
                aabb.center(),
                0.,
                Vec2::new(aabb.width(), aabb.height()),
                Color::GREEN,
            );
        });
    }
}

#[cfg(feature = "algorithm")]
pub fn draw_path(mut gizmos: Gizmos, path_query: Query<&Path>, tilemaps: Query<&Tilemap>) {
    for path in path_query.iter() {
        let tilemap = tilemaps.get(path.get_target_tilemap()).unwrap();

        for node in path.iter() {
            gizmos.circle_2d(tilemap.index_to_world(*node), 10., Color::YELLOW_GREEN);
        }
    }
}

pub fn draw_axis(mut gizmos: Gizmos) {
    gizmos.line_2d(Vec2::NEG_X * 1e10, Vec2::X * 1e10, Color::RED);
    gizmos.line_2d(Vec2::NEG_Y * 1e10, Vec2::Y * 1e10, Color::GREEN);
}

#[cfg(feature = "debug")]
pub fn draw_grid(mut gizmos: Gizmos) {
    const SIZE: f32 = 256.;

    for y in -100..100 {
        gizmos.line_2d(
            Vec2::new(-100. * SIZE, y as f32 * SIZE),
            Vec2::new(100. * SIZE, y as f32 * SIZE),
            Color::WHITE,
        );
    }

    for x in -100..100 {
        gizmos.line_2d(
            Vec2::new(x as f32 * SIZE, -100. * SIZE),
            Vec2::new(x as f32 * SIZE, 100. * SIZE),
            Color::WHITE,
        );
    }
}

#[cfg(feature = "debug")]
pub fn draw_tiles(mut gizmos: Gizmos, tiles: Query<&Tile>, tilemaps: Query<&Tilemap>) {
    for tile in tiles.iter() {
        let tilemap = tilemaps.get(tile.tilemap_id).unwrap();
        let center = tilemap.index_to_world(tile.index);
        gizmos.rect_2d(center, 0., Vec2::new(8., 8.), Color::YELLOW);
    }
}
