use bevy::{prelude::*, render::primitives::Frustum};

use crate::tilemap::{Tile, TileMap, TileMapBundle};

pub fn random_tests() {}

// pub fn test_create_tilemap(mut commands: Commands, asset_server: Res<AssetServer>) {
//     commands.spawn(TileMapBundle {
//         tilemap: TileMap {
//             coord_system: crate::tilemap::TileMapCoordSystem::Square,
//             size: UVec2::new(4, 4),
//             tile_size: UVec2::new(16, 16),
//             textures: vec![],
//             sprite_slicing: crate::tilemap::TileSpriteMode::SingleAtlas(
//                 asset_server.load("test/test/png"),
//             ),
//         },
//         ..default()
//     });
// }
