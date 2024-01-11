/*
 * The visualization seems incorrect.
 * But the data of the result is correct.
 */

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
    math::TileArea,
    tilemap::{
        algorithm::path::{PathTile, PathTilemap},
        bundles::TilemapBundle,
        map::{
            TileRenderSize, TilemapRotation, TilemapSlotSize, TilemapStorage, TilemapTexture,
            TilemapTextureDescriptor, TilemapType,
        },
        tile::{TileBuilder, TileLayer},
    },
    EntiTilesPlugin,
};
use helpers::EntiTilesHelpersPlugin;

mod helpers;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EntiTilesPlugin, EntiTilesHelpersPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, assets_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let entity = commands.spawn_empty().id();
    let mut tilemap = TilemapBundle {
        tile_render_size: TileRenderSize(Vec2::new(32., 16.)),
        slot_size: TilemapSlotSize(Vec2::new(32., 16.)),
        ty: TilemapType::Isometric,
        storage: TilemapStorage::new(64, entity),
        texture: TilemapTexture::new(
            assets_server.load("test_isometric.png"),
            TilemapTextureDescriptor::new(
                UVec2 { x: 32, y: 32 },
                UVec2 { x: 32, y: 16 },
                FilterMode::Nearest,
            ),
            TilemapRotation::None,
        ),
        ..Default::default()
    };

    tilemap.storage.fill_rect(
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
            tilemap: entity,
            custom_weight: None,
            max_step: None,
        },
        // Uncomment this to use async pathfinder
        // AsyncPathfinder {
        //     max_step_per_frame: 400,
        // },
    ));

    commands.entity(entity).insert((tilemap, path_tilemap));
}
