use bevy::prelude::*;

use crate::tilemap::*;

pub fn random_tests(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(Tilemap::from(TilemapBuilder::new(
        156315536,
        TileType::Square,
        UVec2::new(8, 8),
        UVec2::new(16, 16),
        asset_server.load("test/test.png"),
    )));
}

// pub fn test_create_tilemap(mut commands: Commands, asset_server: Res<AssetServer>) {
//     commands.spawn(TilemapBundle {
//         tilemap: Tilemap {
//             coord_system: crate::tilemap::TilemapCoordSystem::Square,
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
