use bevy::{
    prelude::{App, AssetServer, Camera2dBundle, Commands, Res, Startup, UVec2, Vec2},
    render::render_resource::FilterMode,
    DefaultPlugins,
};
use bevy_entitiles::{
    math::FillArea,
    render::texture::TilemapTextureDescriptor,
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
        TileType::IsometricDiamond,
        UVec2 { x: 20, y: 10 },
        Vec2 { x: 32., y: 16. },
        "test_map".to_string(),
    )
    .with_texture(
        assets_server.load("test_nonuniform.png"),
        TilemapTextureDescriptor {
            size: UVec2 { x: 64, y: 80 },
            tiles_uv: vec![
                (UVec2 { x: 0, y: 0 }, UVec2 { x: 32, y: 32 }).into(),
                (UVec2 { x: 32, y: 8 }, UVec2 { x: 64, y: 32 }).into(),
                (UVec2 { x: 0, y: 32 }, UVec2 { x: 32, y: 59 }).into(),
                (UVec2 { x: 32, y: 32 }, UVec2 { x: 52, y: 69 }).into(),
                (UVec2 { x: 0, y: 64 }, UVec2 { x: 32, y: 80 }).into(),
            ],
            filter_mode: FilterMode::Nearest,
            is_uniform: false,
        },
    )
    .build(&mut commands);

    tilemap.fill_rect(
        &mut commands,
        FillArea::full(&tilemap),
        &TileBuilder::new(4),
    );

    tilemap.fill_rect(
        &mut commands,
        FillArea::new(UVec2 { x: 2, y: 2 }, Some(UVec2 { x: 10, y: 7 }), &tilemap),
        &TileBuilder::new(1),
    );

    tilemap.fill_rect(
        &mut commands,
        FillArea::new(UVec2 { x: 15, y: 5 }, Some(UVec2 { x: 4, y: 2 }), &tilemap),
        &TileBuilder::new(2),
    );

    tilemap.set(&mut commands, UVec2 { x: 5, y: 5 }, &TileBuilder::new(3));
    tilemap.set(&mut commands, UVec2 { x: 2, y: 6 }, &TileBuilder::new(0));

    commands.entity(tilemap_entity).insert(tilemap);
}
