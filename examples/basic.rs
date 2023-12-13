use bevy::{
    math::Vec4,
    prelude::{App, AssetServer, Camera2dBundle, Commands, Res, Startup, UVec2, Vec2},
    render::{color::Color, render_resource::FilterMode},
    DefaultPlugins,
};
use bevy_entitiles::{
    math::FillArea,
    render::texture::{TilemapTexture, TilemapTextureDescriptor},
    tilemap::{
        map::TilemapBuilder,
        tile::{TileBuilder, TileFlip, TileType},
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
        TilemapTextureDescriptor {
            size: UVec2 { x: 32, y: 32 },
            tile_size: UVec2 { x: 16, y: 16 },
            filter_mode: FilterMode::Nearest,
        },
    ))
    .build(&mut commands);

    tilemap.fill_rect(
        &mut commands,
        FillArea::full(&tilemap),
        &TileBuilder::new().with_layer(0, 0),
    );

    tilemap.fill_rect(
        &mut commands,
        FillArea::new(UVec2 { x: 2, y: 2 }, Some(UVec2 { x: 10, y: 7 }), &tilemap),
        &TileBuilder::new()
            .with_layer(0, 1)
            .with_color(Vec4::new(0.8, 1., 0.8, 0.5)),
    );

    tilemap.set(
        &mut commands,
        UVec2 { x: 18, y: 8 },
        &TileBuilder::new()
            .with_layer(0, 0)
            .with_color(Color::BLUE.into()),
    );

    tilemap.set(
        &mut commands,
        UVec2 { x: 1, y: 1 },
        &TileBuilder::new()
            .with_layer(0, 1)
            .with_flip(TileFlip::Horizontal),
    );

    tilemap.set(
        &mut commands,
        UVec2 { x: 1, y: 2 },
        &TileBuilder::new()
            .with_layer(0, 1)
            .with_flip(TileFlip::Vertical),
    );

    tilemap.set(
        &mut commands,
        UVec2 { x: 1, y: 3 },
        &TileBuilder::new()
            .with_layer(0, 1)
            .with_flip(TileFlip::Both),
    );

    tilemap.update_rect(
        &mut commands,
        FillArea::new(UVec2 { x: 1, y: 3 }, Some(UVec2 { x: 3, y: 3 }), &tilemap),
        1,
        Some(3),
    );

    commands.entity(tilemap_entity).insert(tilemap);

    let (tilemap_entity, mut tilemap) = TilemapBuilder::new(
        TileType::IsometricDiamond,
        UVec2 { x: 20, y: 10 },
        Vec2 { x: 32., y: 16. },
        "test_map".to_string(),
    )
    .with_texture(TilemapTexture::new(
        assets_server.load("test_isometric.png"),
        TilemapTextureDescriptor {
            size: UVec2 { x: 32, y: 32 },
            tile_size: UVec2 { x: 32, y: 16 },
            filter_mode: FilterMode::Nearest,
        },
    ))
    .with_translation(Vec2 { x: -400., y: 0. })
    .build(&mut commands);

    tilemap.fill_rect(
        &mut commands,
        FillArea::full(&tilemap),
        &TileBuilder::new().with_layer(0, 0),
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
        &TileBuilder::new()
            .with_layer(0, 0)
            .with_color(Vec4::new(1., 1., 0., 1.)),
    );

    commands.entity(tilemap_entity).insert(tilemap);
}
