use bevy::{prelude::*, render::render_resource::FilterMode};

use crate::{render::texture::TilemapTextureDescriptor, tilemap::*};

pub fn random_tests(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            scale: 1.,
            ..default()
        },
        ..default()
    });
    TilemapBuilder::new(
        TileType::IsometricDiamond,
        UVec2::new(20, 20),
        Vec2::new(30., 30.),
    )
    .with_center(Vec2::ZERO)
    // .with_texture(
    //     asset_server.load("test/test.png"),
    //     TilemapTextureDescriptor {
    //         tile_count: UVec2::new(2, 2),
    //         tile_size: UVec2::new(16, 16),
    //         filter_mode: FilterMode::Nearest,
    //     },
    // )
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
                &TileBuilder::new(UVec2::ZERO, 0).with_color(Color::WHITE.into()),
            );
            tilemap.fill_rect(
                &mut commands,
                UVec2::new(10, 12),
                UVec2::new(5, 5),
                &TileBuilder::new(UVec2::ZERO, 1),
            )
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
