use bevy::{
    app::Update,
    ecs::{
        entity::Entity,
        query::With,
        system::{Query, ResMut},
    },
    input::{keyboard::KeyCode, ButtonInput},
    math::IVec2,
    prelude::{App, AssetServer, Camera2dBundle, Commands, Res, Startup, UVec2, Vec2},
    render::render_resource::FilterMode,
    DefaultPlugins,
};
use bevy_entitiles::{
    algorithm::pathfinding::PathTilemaps,
    math::TileArea,
    serializing::map::{
        load::TilemapLoader,
        save::{TilemapSaver, TilemapSaverMode},
        TilemapLayer,
    },
    tilemap::{
        algorithm::path::{PathTile, PathTilemap},
        bundles::StandardTilemapBundle,
        map::{
            TilePivot, TileRenderSize, TilemapName, TilemapRotation, TilemapSlotSize,
            TilemapStorage, TilemapTexture, TilemapTextureDescriptor, TilemapType,
        },
        physics::{PhysicsTile, PhysicsTilemap},
        tile::{TileBuilder, TileLayer},
    },
    EntiTilesPlugin,
};
use bevy_xpbd_2d::plugins::{PhysicsDebugPlugin, PhysicsPlugins};
use helpers::EntiTilesHelpersPlugin;

mod helpers;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            EntiTilesPlugin,
            PhysicsPlugins::default(),
            PhysicsDebugPlugin::default(),
            EntiTilesHelpersPlugin::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, save_and_load)
        .run();
}

fn setup(
    mut commands: Commands,
    assets_server: Res<AssetServer>,
    mut path_tilemaps: ResMut<PathTilemaps>,
) {
    commands.spawn(Camera2dBundle::default());

    let entity = commands.spawn_empty().id();
    let mut tilemap = StandardTilemapBundle {
        name: TilemapName("test_map".to_string()),
        tile_render_size: TileRenderSize(Vec2::new(32., 16.)),
        slot_size: TilemapSlotSize(Vec2::new(32., 16.)),
        ty: TilemapType::Isometric,
        storage: TilemapStorage::new(64, entity),
        tile_pivot: TilePivot(Vec2 { x: 0.5, y: 0. }),
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
        TileArea::new(IVec2::ZERO, UVec2 { x: 20, y: 20 }),
        TileBuilder::new().with_layer(0, TileLayer::no_flip(0)),
    );

    tilemap.storage.fill_rect(
        &mut commands,
        TileArea::new(IVec2 { x: 2, y: 2 }, UVec2 { x: 10, y: 7 }),
        TileBuilder::new().with_layer(0, TileLayer::no_flip(0)),
    );

    let mut path_tilemap = PathTilemap::new();
    path_tilemap.fill_path_rect_custom(TileArea::new(IVec2::ZERO, UVec2 { x: 20, y: 20 }), |_| {
        Some(PathTile {
            cost: rand::random::<u32>() % 10,
        })
    });
    path_tilemaps.insert(entity, path_tilemap);

    let mut physics_tilemap = PhysicsTilemap::new();
    physics_tilemap.set(
        IVec2 { x: 2, y: 3 },
        PhysicsTile {
            rigid_body: true,
            friction: Some(0.5),
        },
    );
    physics_tilemap.fill_rect(
        TileArea::new(IVec2 { x: 3, y: 4 }, UVec2 { x: 5, y: 4 }),
        PhysicsTile {
            rigid_body: false,
            friction: None,
        },
        true,
    );

    commands.entity(entity).insert((tilemap, physics_tilemap));
}

fn save_and_load(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    tilemap: Query<Entity, With<TilemapStorage>>,
) {
    // save
    if input.just_pressed(KeyCode::Space) {
        for t in tilemap.iter() {
            commands.entity(t).insert(TilemapSaver {
                path: "C:\\saves".to_string(),
                mode: TilemapSaverMode::Tilemap,
                layers: TilemapLayer::all(),
                texture_path: Some("test_isometric.png".to_string()),
                remove_after_save: true,
            });
            println!("Saved tilemap!");
        }
    }

    // load
    if input.just_pressed(KeyCode::AltRight) {
        commands.spawn(TilemapLoader {
            path: "C:\\saves".to_string(),
            map_name: "test_map".to_string(),
            layers: TilemapLayer::all(),
        });
        println!("Loading tilemap...");
    }
}
