use bevy::{
    app::Update,
    ecs::{entity::Entity, system::Query},
    input::{keyboard::KeyCode, Input},
    prelude::{App, AssetServer, Camera2dBundle, Commands, Res, Startup, UVec2, Vec2},
    render::render_resource::FilterMode,
    DefaultPlugins,
};
use bevy_entitiles::{
    algorithm::pathfinding::PathTile,
    math::FillArea,
    render::texture::{TilemapTexture, TilemapTextureDescriptor},
    serializing::{
        load::{TilemapLoadFailure, TilemapLoaderBuilder},
        save::TilemapSaverBuilder,
        TilemapLayer,
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
        .add_systems(Update, (save_and_load, failure_handle))
        .run();
}

fn setup(mut commands: Commands, assets_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let (tilemap_entity, mut tilemap) = TilemapBuilder::new(
        TileType::Isometric,
        UVec2 { x: 20, y: 20 },
        Vec2 { x: 32.0, y: 16.0 },
        "test_map".to_string(),
    )
    .with_texture(TilemapTexture::new(
        assets_server.load("test_isometric.png"),
        TilemapTextureDescriptor::new(
            UVec2 { x: 32, y: 32 },
            UVec2 { x: 32, y: 16 },
            FilterMode::Nearest,
        ),
    ))
    .with_pivot(Vec2 { x: 0.5, y: 0. })
    .with_render_chunk_size(64)
    .build(&mut commands);

    tilemap.fill_rect(
        &mut commands,
        FillArea::full(&tilemap),
        &TileBuilder::new().with_layer(0, 0),
    );

    tilemap.fill_rect(
        &mut commands,
        FillArea::new(UVec2 { x: 2, y: 2 }, Some(UVec2 { x: 10, y: 7 }), &tilemap),
        &TileBuilder::new().with_layer(0, 0),
    );

    let mut path_tilemap = PathTilemap::new(tilemap_entity);
    path_tilemap.fill_path_rect_custom(&tilemap, FillArea::full(&tilemap), |_| PathTile {
        cost: rand::random::<u32>() % 10,
    });

    commands
        .entity(tilemap_entity)
        .insert((tilemap, path_tilemap));
}

fn save_and_load(mut commands: Commands, input: Res<Input<KeyCode>>, tilemap: Query<&Tilemap>) {
    // save
    if input.just_pressed(KeyCode::Space) {
        for t in tilemap.iter() {
            TilemapSaverBuilder::new("C:\\saves\\".to_string())
                .with_layer(TilemapLayer::All)
                .with_texture("test_isometric.png".to_string())
                .remove_map_after_done()
                .build(&mut commands, t.id());
            println!("Saved tilemap!");
        }
    }

    // load
    if input.just_pressed(KeyCode::AltRight) {
        let entity = commands.spawn_empty().id();
        TilemapLoaderBuilder::new("C:\\saves".to_string(), "test_map".to_string())
            .with_layer(TilemapLayer::All)
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
