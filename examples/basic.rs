use bevy::{
    prelude::{App, AssetServer, Camera2dBundle, Commands, Res, Startup, UVec2, Vec2, Update},
    render::render_resource::FilterMode,
    DefaultPlugins,
};
use bevy_entitiles::{
    render::texture::TilemapTextureDescriptor,
    tilemap::{TileBuilder, TileType, TilemapBuilder},
    EntiTilesPlugin, debug::camera_movement::camera_control,
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
        TileType::Square,
        UVec2 { x: 20, y: 10 },
        Vec2 { x: 32.0, y: 32.0 },
    )
    .with_texture(
        assets_server.load("test/test_square.png"),
        TilemapTextureDescriptor {
            tile_count: UVec2 { x: 2, y: 2 },
            tile_size: UVec2 { x: 16, y: 16 },
            filter_mode: FilterMode::Nearest,
        },
    )
    .build(&mut commands);

    tilemap.fill_rect(
        &mut commands,
        UVec2 { x: 0, y: 0 },
        None,
        &TileBuilder::new(UVec2::ZERO, 0),
    );

    tilemap.fill_rect(
        &mut commands,
        UVec2 { x: 2, y: 2 },
        Some(UVec2 { x: 10, y: 7 }),
        &TileBuilder::new(UVec2::ZERO, 1),
    );

    commands.entity(tilemap_entity).insert(tilemap);
}
