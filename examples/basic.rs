use bevy::{
    app::Update,
    ecs::system::ResMut,
    input::{keyboard::KeyCode, Input},
    math::Vec4,
    prelude::{App, AssetServer, Camera2dBundle, Commands, Res, Startup, UVec2, Vec2},
    render::render_resource::FilterMode,
    DefaultPlugins,
};
use bevy_entitiles::{
    math::FillArea,
    render::texture::{TilemapTexture, TilemapTextureDescriptor},
    tilemap::{
        map::TilemapBuilder,
        tile::{TileBuilder, TileType},
    },
    EntiTilesPlugin,
};
use helpers::EntiTilesDebugPlugin;

mod helpers;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EntiTilesPlugin, EntiTilesDebugPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, assets_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let (tilemap_entity, mut tilemap) = TilemapBuilder::new(
        TileType::Square,
        UVec2 { x: 20, y: 10 },
        Vec2 { x: 16., y: 16. },
        "test_map".to_string(),
    )
    .with_texture(TilemapTexture::new(
        assets_server.load("test_square.png"),
        TilemapTextureDescriptor::from_full_grid(
            UVec2 { x: 32, y: 32 },
            UVec2 { x: 2, y: 2 },
            FilterMode::Nearest,
        ),
    ))
    .build(&mut commands);

    tilemap.fill_rect(
        &mut commands,
        FillArea::full(&tilemap),
        &TileBuilder::new(0),
    );

    tilemap.fill_rect(
        &mut commands,
        FillArea::new(UVec2 { x: 2, y: 2 }, Some(UVec2 { x: 10, y: 7 }), &tilemap),
        &TileBuilder::new(1).with_color(Vec4::new(0.8, 1., 0.8, 0.1)),
    );

    tilemap.update_rect(
        &mut commands,
        FillArea::new(UVec2 { x: 1, y: 3 }, Some(UVec2 { x: 3, y: 3 }), &tilemap),
        3,
        Some(2),
    );

    commands.entity(tilemap_entity).insert(tilemap);

    let (tilemap_entity, mut tilemap) = TilemapBuilder::new(
        TileType::Square,
        UVec2 { x: 20, y: 10 },
        Vec2 { x: 16., y: 16. },
        "test_map".to_string(),
    )
    .with_translation(Vec2 { x: 0., y: -300. })
    .build(&mut commands);

    tilemap.fill_rect(
        &mut commands,
        FillArea::full(&tilemap),
        &TileBuilder::new(0).with_color(Vec4::new(1., 1., 0., 1.)),
    );

    commands.entity(tilemap_entity).insert(tilemap);
}
