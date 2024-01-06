use bevy::{
    app::Update,
    ecs::{entity::Entity, system::Query},
    input::{keyboard::KeyCode, Input},
    math::IVec2,
    prelude::{App, AssetServer, Camera2dBundle, Commands, Res, Startup, UVec2, Vec2},
    render::render_resource::FilterMode,
    DefaultPlugins,
};
use bevy_entitiles::{
    debug::EntiTilesDebugPlugin,
    math::TileArea,
    render::texture::{TilemapTexture, TilemapTextureDescriptor},
    serializing::{
        load::{TilemapLoadFailure, TilemapLoaderBuilder},
        save::TilemapSaverBuilder,
        TilemapLayer,
    },
    tilemap::{
        algorithm::path::{PathTile, PathTilemap},
        layer::TileLayer,
        map::{Tilemap, TilemapBuilder, TilemapRotation},
        tile::{TileBuilder, TilemapType},
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
            EntiTilesDebugPlugin,
            EntiTilesHelpersPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (save_and_load, failure_handle))
        .run();
}

fn setup(mut commands: Commands, assets_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let mut tilemap = TilemapBuilder::new(
        TilemapType::Isometric,
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
        TilemapRotation::None,
    ))
    .with_pivot(Vec2 { x: 0.5, y: 0. })
    .with_chunk_size(64)
    .build(&mut commands);

    tilemap.fill_rect(
        &mut commands,
        TileArea::new(IVec2::ZERO, UVec2 { x: 20, y: 20 }),
        TileBuilder::new().with_layer(0, TileLayer::new().with_texture_index(0)),
    );

    tilemap.fill_rect(
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

    commands
        .entity(tilemap.id())
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
