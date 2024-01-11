use bevy::{
    app::Update,
    ecs::{entity::Entity, query::With, system::Query},
    input::{keyboard::KeyCode, Input},
    math::IVec2,
    prelude::{App, AssetServer, Camera2dBundle, Commands, Res, Startup, UVec2, Vec2},
    render::render_resource::FilterMode,
    DefaultPlugins,
};
use bevy_entitiles::{
    math::TileArea,
    serializing::map::{
        load::{TilemapLoadFailure, TilemapLoaderBuilder},
        save::TilemapSaver,
        TilemapLayer,
    },
    tilemap::{
        algorithm::path::{PathTile, PathTilemap},
        bundles::TilemapBundle,
        map::{
            TilePivot, TileRenderSize, TilemapName, TilemapRotation, TilemapSlotSize,
            TilemapStorage, TilemapTexture, TilemapTextureDescriptor, TilemapType,
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
        .add_systems(Update, (save_and_load, failure_handle))
        .run();
}

fn setup(mut commands: Commands, assets_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let entity = commands.spawn_empty().id();
    let mut tilemap = TilemapBundle {
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
        TileBuilder::new().with_layer(0, TileLayer::new().with_texture_index(0)),
    );

    tilemap.storage.fill_rect(
        &mut commands,
        TileArea::new(IVec2 { x: 2, y: 2 }, UVec2 { x: 10, y: 7 }),
        TileBuilder::new().with_layer(0, TileLayer::new().with_texture_index(0)),
    );

    let mut path_tilemap = PathTilemap::new();
    path_tilemap.fill_path_rect_custom(TileArea::new(IVec2::ZERO, UVec2 { x: 20, y: 20 }), |_| {
        Some(PathTile {
            cost: rand::random::<u32>() % 10,
        })
    });

    commands.entity(entity).insert((tilemap, path_tilemap));
}

fn save_and_load(
    mut commands: Commands,
    input: Res<Input<KeyCode>>,
    tilemap: Query<Entity, With<TilemapStorage>>,
) {
    // save
    if input.just_pressed(KeyCode::Space) {
        for t in tilemap.iter() {
            commands.entity(t).insert(
                TilemapSaver::new("C:\\saves".to_string())
                    .with_layer(TilemapLayer::Color)
                    .with_layer(TilemapLayer::Path)
                    .with_texture("test_isometric.png".to_string())
                    .remove_after_save(),
            );
            println!("Saved tilemap!");
        }
    }

    // load
    if input.just_pressed(KeyCode::AltRight) {
        let entity = commands.spawn_empty().id();
        TilemapLoaderBuilder::new("C:\\saves".to_string(), "test_map".to_string())
            .with_layer(TilemapLayer::Color)
            .with_layer(TilemapLayer::Path)
            .build(&mut commands, entity);
        println!("Loading tilemap...");
    }
}

fn failure_handle(mut commands: Commands, errs: Query<(Entity, &TilemapLoadFailure)>) {
    for (entity, err) in errs.iter() {
        println!(
            "Failed to load tilemap: {}\\{}",
            err.path.clone(),
            &err.map_name
        );
        commands.entity(entity).remove::<TilemapLoadFailure>();
    }
}
