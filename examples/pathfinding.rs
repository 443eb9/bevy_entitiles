use bevy::{
    prelude::{App, AssetServer, Camera2dBundle, Commands, Res, Startup, UVec2, Vec2},
    render::render_resource::FilterMode,
    DefaultPlugins,
};
use bevy_entitiles::{
    algorithm::pathfinding::{AsyncPathfinder, PathTile, Pathfinder},
    math::FillArea,
    render::texture::{TilemapTexture, TilemapTextureDescriptor},
    tilemap::{
        algorithm::path::PathTilemap,
        map::{TilemapBuilder, TilemapRotation},
        tile::{TileBuilder, TileLayer, TileType},
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

    let mut tilemap = TilemapBuilder::new(
        TileType::Isometric,
        UVec2 { x: 500, y: 500 },
        Vec2 { x: 32., y: 16. },
        "test_map".to_string(),
    )
    .with_texture(TilemapTexture::new(
        assets_server.load("test_isometric.png"),
        TilemapTextureDescriptor::new(
            UVec2 { x: 32, y: 32 },
            UVec2 { x: 32, y: 16 },
            FilterMode::Nearest,
        ),
        TilemapRotation::None,
    ))
    .with_pivot(Vec2 { x: 0.5, y: 0. })
    .with_render_chunk_size(64)
    .build(&mut commands);

    tilemap.fill_rect(
        FillArea::full(&tilemap),
        TileBuilder::new().with_layer(0, TileLayer::new().with_texture_index(0)),
    );

    let mut path_tilemap = PathTilemap::new(tilemap.id());
    path_tilemap.fill_path_rect_custom(&tilemap, FillArea::full(&tilemap), |_| PathTile {
        cost: rand::random::<u32>() % 10,
    });

    commands.spawn_empty().insert((
        Pathfinder {
            origin: UVec2 { x: 0, y: 0 },
            dest: UVec2 { x: 499, y: 499 },
            allow_diagonal: false,
            tilemap: tilemap.id(),
            custom_weight: None,
            max_step: None,
        },
        // remove the AsyncPathfinder if you want to synchronize the pathfinding
        AsyncPathfinder {
            max_step_per_frame: 300,
        },
    ));

    commands
        .entity(tilemap.id())
        .insert((tilemap, path_tilemap));
}
