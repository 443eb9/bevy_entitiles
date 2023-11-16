use bevy::{
    prelude::{App, AssetServer, Camera2dBundle, Commands, Res, Startup, UVec2, Update, Vec2},
    render::render_resource::FilterMode,
    DefaultPlugins,
};
use bevy_entitiles::{
    algorithm::pathfinding::{PathTile, Pathfinder},
    debug::camera_movement::camera_control,
    math::FillArea,
    render::texture::TilemapTextureDescriptor,
    tilemap::{TileBuilder, TileType, TilemapBuilder},
    EntiTilesPlugin,
};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EntiTilesPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, camera_control)
        .run();
}

fn setup(mut commands: Commands, assets_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let (tilemap_entity, mut tilemap) = TilemapBuilder::new(
        TileType::IsometricDiamond,
        UVec2 { x: 20, y: 10 },
        Vec2 { x: 32.0, y: 16.0 },
    )
    .with_texture(
        assets_server.load("test/test_isometric.png"),
        TilemapTextureDescriptor {
            tile_count: UVec2 { x: 1, y: 2 },
            tile_size: UVec2 { x: 32, y: 16 },
            filter_mode: FilterMode::Nearest,
        },
    )
    .build(&mut commands);

    tilemap.fill_rect(
        &mut commands,
        FillArea::full(&tilemap),
        &TileBuilder::new(UVec2::ZERO, 0),
    );

    tilemap.fill_rect(
        &mut commands,
        FillArea::new(UVec2 { x: 2, y: 2 }, Some(UVec2 { x: 10, y: 7 }), &tilemap),
        &TileBuilder::new(UVec2::ZERO, 1),
    );

    tilemap.fill_path_rect_custom(&mut commands, FillArea::full(&tilemap), |_| PathTile {
        cost: rand::random::<u32>() % 10,
    });

    commands.entity(tilemap_entity).insert(tilemap);

    commands.spawn_empty().insert(Pathfinder {
        origin: UVec2 { x: 0, y: 0 },
        dest: UVec2 { x: 10, y: 9 },
        allow_diagonal: true,
        tilemap: tilemap_entity,
        custom_weight: None,
        max_step: Some(200),
    });
}
