use bevy::{
    app::Update,
    ecs::system::Query,
    input::{keyboard::KeyCode, Input},
    prelude::{App, AssetServer, Camera2dBundle, Commands, Res, Startup, UVec2, Vec2},
    render::render_resource::FilterMode,
    DefaultPlugins,
};
use bevy_entitiles::{
    algorithm::pathfinding::PathTile,
    math::FillArea,
    render::texture::TilemapTextureDescriptor,
    serializing::{
        save::TilemapSaverBuilder, TilemapLayer,
    },
    tilemap::{
        algorithm::path::PathTilemap,
        map::{Tilemap, TilemapBuilder},
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
        .add_systems(Update, save)
        .run();
}

fn setup(mut commands: Commands, assets_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let (tilemap_entity, mut tilemap) = TilemapBuilder::new(
        TileType::IsometricDiamond,
        UVec2 { x: 500, y: 500 },
        Vec2 { x: 32.0, y: 16.0 },
    )
    .with_texture(
        assets_server.load("test_isometric.png"),
        TilemapTextureDescriptor::from_full_grid(
            UVec2 { x: 32, y: 32 },
            UVec2 { x: 1, y: 2 },
            FilterMode::Nearest,
        ),
    )
    .with_anchor(Vec2 { x: 0.5, y: 0. })
    .with_render_chunk_size(64)
    .build(&mut commands);

    tilemap.fill_rect(
        &mut commands,
        FillArea::full(&tilemap),
        &TileBuilder::new(0),
    );

    tilemap.fill_rect(
        &mut commands,
        FillArea::new(UVec2 { x: 2, y: 2 }, Some(UVec2 { x: 10, y: 7 }), &tilemap),
        &TileBuilder::new(1),
    );

    let mut path_tilemap = PathTilemap::new(tilemap_entity);
    path_tilemap.fill_path_rect_custom(&tilemap, FillArea::full(&tilemap), |_| PathTile {
        cost: rand::random::<u32>() % 10,
    });

    commands
        .entity(tilemap_entity)
        .insert((tilemap, path_tilemap));
}

fn save(mut commands: Commands, input: Res<Input<KeyCode>>, tilemap: Query<&Tilemap>) {
    if input.just_pressed(KeyCode::Space) {
        for t in tilemap.iter() {
            TilemapSaverBuilder::new("C:\\mytilemap".to_string())
                .with_layer(TilemapLayer::Texture)
                .with_layer(TilemapLayer::Algorithm)
                .remove_map_after_done()
                .build(&mut commands, t.id());
        }
    }
}
