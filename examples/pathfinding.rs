use bevy::{
    math::IVec2,
    prelude::{App, AssetServer, Camera2dBundle, Commands, Res, Startup, UVec2, Vec2},
    render::render_resource::FilterMode,
    DefaultPlugins,
};
#[allow(unused_imports)]
use bevy_entitiles::algorithm::pathfinding::AsyncPathfinder;
use bevy_entitiles::{
    algorithm::pathfinding::Pathfinder,
    debug::EntiTilesDebugPlugin,
    math::TileArea,
    render::texture::{TilemapTexture, TilemapTextureDescriptor},
    tilemap::{
        algorithm::path::{PathTile, PathTilemap},
        layer::TileLayer,
        map::{TilemapBuilder, TilemapRotation},
        tile::{TileBuilder, TileType},
    },
    EntiTilesPlugin,
};
use helpers::EntiTilesHelpersPlugin;

mod helpers;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            EntiTilesPlugin,
            EntiTilesHelpersPlugin,
            EntiTilesDebugPlugin,
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, assets_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let mut tilemap = TilemapBuilder::new(
        TileType::Isometric,
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
    .with_chunk_size(64)
    .build(&mut commands);

    tilemap.fill_rect(
        &mut commands,
        TileArea::new(IVec2::ZERO, UVec2 { x: 500, y: 500 }),
        TileBuilder::new().with_layer(0, TileLayer::new().with_texture_index(0)),
    );

    let mut path_tilemap = PathTilemap::new();
    path_tilemap.fill_path_rect_custom(
        TileArea::new(IVec2::ZERO, UVec2 { x: 500, y: 500 }),
        |_| {
            Some(PathTile {
                cost: rand::random::<u32>() % 10,
            })
        },
    );

    commands.spawn_empty().insert((
        Pathfinder {
            origin: IVec2 { x: 0, y: 0 },
            dest: IVec2 { x: 499, y: 499 },
            allow_diagonal: false,
            tilemap: tilemap.id(),
            custom_weight: None,
            max_step: None,
        },
        // Uncomment this to use async pathfinder
        // AsyncPathfinder {
        //     max_step_per_frame: 400,
        // },
    ));

    commands
        .entity(tilemap.id())
        .insert((tilemap, path_tilemap));
}
