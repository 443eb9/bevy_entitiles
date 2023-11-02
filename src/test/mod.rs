use bevy::prelude::*;

use crate::tilemap::*;

pub fn random_tests(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            scale: 1.,
            ..default()
        },
        ..default()
    });
    TilemapBuilder::new(
        TileType::Square,
        UVec2::new(20, 20),
        UVec2::new(16, 16),
        Vec2::new(30., 30.),
        asset_server.load("test/test.png"),
    )
    .with_center(Vec2::ZERO)
    .build(&mut commands);
}

static mut FLAG: bool = false;
pub fn set_tile(mut commands: Commands, mut tilemap: Query<&mut Tilemap>) {
    unsafe {
        if FLAG {
            return;
        }
        for mut tilemap in tilemap.iter_mut() {
            tilemap.fill_rect(
                &mut commands,
                UVec2::ZERO,
                UVec2::new(20, 20),
                &TileBuilder::new(UVec2::ZERO, 0).with_color(Vec4::new(1., 1., 1., 1.)),
            );
        }
        FLAG = true;
    }
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
