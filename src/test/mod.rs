use bevy::prelude::*;

use crate::tilemap::*;

pub fn random_tests(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());
    TilemapBuilder::new(
        TileType::Square,
        UVec2::new(8, 8),
        UVec2::new(16, 16),
        asset_server.load("test/test.png"),
    )
    .build(&mut commands);
    commands.spawn(ColorMesh2dBundle {
        mesh: meshes
            .add(shape::Quad::new(Vec2::new(50., 50.)).into())
            .into(),
        material: materials.add(Color::WHITE.into()),
        transform: Transform::from_xyz(0., 0., -1.),
        ..default()
    });
}
static mut FLAG: bool = false;
pub fn set_tile(mut commands: Commands, mut tilemap: Query<&mut Tilemap>) {
    unsafe {
        if FLAG {
            return;
        }
        for mut tilemap in tilemap.iter_mut() {
            tilemap.set(&mut commands, TileBuilder::new(UVec2::ZERO, 0));
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
