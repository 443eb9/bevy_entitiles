use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_real_timer};
use bevy_entitiles::prelude::*;
use helpers::EntiTilesHelpersPlugin;

mod helpers;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            EntiTilesPlugin,
            EntiTilesHelpersPlugin { inspector: false },
        ))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            detect.run_if(on_real_timer(Duration::from_millis(100))),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    assets_server: Res<AssetServer>,
    mut path_tilemaps: ResMut<PathTilemaps>,
    mut materials: ResMut<Assets<StandardTilemapMaterial>>,
    mut textures: ResMut<Assets<TilemapTextures>>,
) {
    bevy::log::info!("===================================");
    bevy::log::info!("Loading tilemap, please be patient.");
    bevy::log::info!("===================================");

    commands.spawn(Camera2dBundle::default());

    let entity = commands.spawn_empty().id();
    let mut tilemap = StandardTilemapBundle {
        tile_render_size: TileRenderSize(Vec2::new(32., 16.)),
        slot_size: TilemapSlotSize(Vec2::new(32., 16.)),
        ty: TilemapType::Isometric,
        storage: TilemapStorage::new(64, entity),
        material: materials.add(StandardTilemapMaterial::default()),
        textures: textures.add(TilemapTextures::single(
            TilemapTexture::new(
                assets_server.load("test_isometric.png"),
                TilemapTextureDescriptor::new(UVec2 { x: 32, y: 32 }, UVec2 { x: 32, y: 16 }),
            ),
            FilterMode::Nearest,
        )),
        ..Default::default()
    };

    tilemap.storage.fill_rect(
        &mut commands,
        GridRect::new(IVec2::ZERO, UVec2 { x: 500, y: 500 }),
        TileBuilder::new().with_layer(0, TileLayer::no_flip(0)),
    );

    let mut path_tilemap = PathTilemap::new();
    path_tilemap.fill_path_rect_custom(
        GridRect::new(IVec2::ZERO, UVec2 { x: 1000, y: 1000 }),
        |_| {
            Some(PathTile {
                cost: rand::random::<u32>() % 10,
            })
        },
    );
    path_tilemaps.insert(entity, path_tilemap);

    let queue = (0..100).into_iter().map(|_| {
        (
            commands.spawn_empty().id(),
            PathFinder {
                origin: IVec2::ZERO,
                dest: IVec2::splat(499),
                allow_diagonal: false,
                max_steps: None,
            },
        )
    });

    let pathfinding_queue = PathFindingQueue::new_with_schedules(queue);

    commands.entity(entity).insert((tilemap, pathfinding_queue));
}

fn detect(queues_query: Query<&PathFindingQueue>) {
    queues_query.iter().for_each(|q| {
        if q.is_empty() {
            println!("Pathfinding tasks done!");
        }
    });
}
