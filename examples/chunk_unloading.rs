#![allow(unused_imports)]
use bevy::{
    app::{App, PluginGroup, Startup, Update},
    asset::AssetServer,
    core_pipeline::core_2d::Camera2dBundle,
    ecs::{
        entity::Entity,
        event::{Event, EventReader},
        query::With,
        system::{Commands, Query, Res, ResMut},
    },
    math::{IVec2, UVec2, Vec2},
    render::render_resource::FilterMode,
    utils::HashSet,
    window::{Window, WindowPlugin},
    DefaultPlugins,
};
use bevy_entitiles::{
    debug::{CameraAabbScale, EntiTilesDebugPlugin},
    serializing::{
        chunk::{
            load::{ChunkLoadCache, ChunkLoadConfig},
            save::{self, ChunkSaveCache, ChunkSaveConfig},
        },
        map::TilemapLayer,
    },
    tilemap::{
        buffers::TileBuilderBuffer,
        bundles::TilemapBundle,
        chunking::camera::{CameraChunkUpdater, CameraChunkUpdation},
        map::{
            TileRenderSize, TilemapName, TilemapRotation, TilemapSlotSize, TilemapStorage,
            TilemapTexture, TilemapTextureDescriptor, TilemapType,
        },
        tile::{TileBuilder, TileLayer},
    },
    EntiTilesPlugin,
};
use helpers::EntiTilesHelpersPlugin;

mod helpers;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    present_mode: bevy::window::PresentMode::Immediate,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            EntiTilesPlugin,
            EntiTilesHelpersPlugin,
            EntiTilesDebugPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, on_update)
        .register_type::<HashSet<IVec2>>()
        .insert_resource(ChunkSaveConfig {
            path: "C:\\maps".to_string(),
            chunks_per_frame: 1,
        })
        .insert_resource(ChunkLoadConfig {
            path: "C:\\maps".to_string(),
            chunks_per_frame: 1,
        })
        .insert_resource(CameraAabbScale(Vec2::splat(0.3)))
        .run();
}

#[derive(Event, Debug, Clone, Copy)]
struct GenerateChunk(IVec2);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((Camera2dBundle::default(), CameraChunkUpdater::new(1.3, 2.2)));

    let entity = commands.spawn_empty().id();
    let mut tilemap = TilemapBundle {
        name: TilemapName("infinite_map".to_string()),
        tile_render_size: TileRenderSize(Vec2::new(16., 16.)),
        slot_size: TilemapSlotSize(Vec2::new(16., 16.)),
        ty: TilemapType::Square,
        storage: TilemapStorage::new(16, entity),
        texture: TilemapTexture::new(
            asset_server.load("test_square.png"),
            TilemapTextureDescriptor::new(
                UVec2 { x: 32, y: 32 },
                UVec2 { x: 16, y: 16 },
                FilterMode::Nearest,
            ),
            TilemapRotation::None,
        ),
        ..Default::default()
    };

    // tilemap.storage.fill_rect(
    //     &mut commands,
    //     TileArea::new(IVec2 { x: -100, y: -100 }, UVec2 { x: 200, y: 200 }),
    //     TileBuilder::new().with_layer(0, TileLayer::new().with_texture_index(0)),
    // );

    tilemap.storage.get_storage_raw().reserve_many(
        (-7..=6)
            .into_iter()
            .flat_map(move |x| (-7..=6).into_iter().map(move |y| IVec2 { x, y })),
    );

    commands.entity(entity).insert(tilemap);
}

fn on_update(
    mut commands: Commands,
    mut ev: EventReader<CameraChunkUpdation>,
    tilemap: Query<Entity, With<TilemapStorage>>,
    mut load_cache: ResMut<ChunkLoadCache>,
    mut save_cache: ResMut<ChunkSaveCache>,
) {
    let tilemap = tilemap.single();
    let mut to_load = Vec::new();
    let mut to_unload = Vec::new();

    ev.read().for_each(|e| match e {
        CameraChunkUpdation::Entered(_, chunk) => to_load.push(*chunk),
        CameraChunkUpdation::Left(_, chunk) => to_unload.push((*chunk, true)),
    });

    if !to_load.is_empty() {
        // println!("Loading chunks: {:?}", to_load);
        load_cache.schedule_many(
            &mut commands,
            tilemap,
            TilemapLayer::Color,
            to_load.into_iter(),
        );
    }

    if !to_unload.is_empty() {
        // println!("Unloading chunks: {:?}", to_unload);
        save_cache.schedule_many(
            &mut commands,
            tilemap,
            TilemapLayer::Color,
            to_unload.into_iter(),
        );
    }
}
