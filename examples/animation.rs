use bevy::{
    app::{App, Startup, Update},
    asset::AssetServer,
    core_pipeline::core_2d::Camera2dBundle,
    ecs::system::{Commands, Res},
    math::{UVec2, Vec2},
    render::render_resource::FilterMode,
    DefaultPlugins,
};
use bevy_entitiles::{
    debug::camera_movement::camera_control,
    math::FillArea,
    render::texture::TilemapTextureDescriptor,
    tilemap::{
        map::TilemapBuilder,
        tile::{TileAnimation, TileBuilder, TileType},
    },
    EntiTilesPlugin,
};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EntiTilesPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, camera_control)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let (tilemap_entity, mut tilemap) = TilemapBuilder::new(
        TileType::Square,
        UVec2 { x: 20, y: 20 },
        Vec2 { x: 32., y: 32. },
    )
    .with_texture(
        asset_server.load("test/test_square.png"),
        TilemapTextureDescriptor {
            tile_count: UVec2 { x: 2, y: 2 },
            tile_size: UVec2 { x: 16, y: 16 },
            filter_mode: FilterMode::Nearest,
        },
    )
    .build(&mut commands);

    tilemap.fill_rect(
        &mut commands,
        FillArea::full(&tilemap),
        &TileBuilder::new(UVec2::ZERO, 0).with_animation(TileAnimation {
            sequence: vec![0, 1, 2, 3],
            fps: 5.,
            is_loop: true,
        }),
    );

    tilemap.fill_rect(
        &mut commands,
        FillArea::new(UVec2::ZERO, Some(UVec2 { x: 10, y: 10 }), &tilemap),
        &TileBuilder::new(UVec2::ZERO, 0).with_animation(TileAnimation {
            sequence: vec![0, 1, 2, 3],
            fps: 2.,
            is_loop: true,
        }),
    );

    tilemap.fill_rect(
        &mut commands,
        FillArea::new(UVec2::ZERO, Some(UVec2 { x: 5, y: 5 }), &tilemap),
        &TileBuilder::new(UVec2::ZERO, 0).with_animation(TileAnimation {
            sequence: vec![0, 1, 2, 3],
            fps: 1.,
            is_loop: false,
        }),
    );

    commands.entity(tilemap_entity).insert(tilemap);
}
