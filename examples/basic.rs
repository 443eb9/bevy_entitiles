use bevy::{
    prelude::{
        App, AssetServer, Camera2dBundle, Commands, Res, Startup, UVec2, Update, Vec2, Vec4,
    },
    render::render_resource::FilterMode,
    DefaultPlugins,
};
use bevy_entitiles::{
    debug::camera_movement::camera_control,
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

    // A tilemap with texture
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

    // A tilemap without texture
    let (tilemap_entity, mut tilemap) = TilemapBuilder::new(
        TileType::Square,
        UVec2 { x: 20, y: 10 },
        Vec2 { x: 32.0, y: 32.0 },
    )
    .with_translation(Vec2 { x: 0., y: -300. })
    .build(&mut commands);

    tilemap.fill_rect(
        &mut commands,
        UVec2::ZERO,
        None,
        &TileBuilder::new(UVec2::ZERO, 0).with_color(Vec4::new(1., 0.7, 0., 0.2)),
    );

    commands.entity(tilemap_entity).insert(tilemap);
}
