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
    debug::CameraAabbScale,
    render::culling::FrustumCulling,
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
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    present_mode: bevy::window::PresentMode::Immediate,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            EntiTilesPlugin,
            EntiTilesHelpersPlugin { inspector: false },
            PhysicsPlugins::default(),
            PhysicsDebugPlugin::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, on_update)
        .insert_resource(ChunkSaveConfig {
            path: "C:\\saves".to_string(),
            chunks_per_frame: 1,
        })
        .insert_resource(ChunkLoadConfig {
            path: "C:\\saves".to_string(),
            chunks_per_frame: 1,
        })
        // We need to disable frustum culling to see the load/save process.
        // Otherwise the chunks will be invisible when they are not intersected with the camera aabb
        // and only leaves the green aabb outline.
        // You don't need to do this in your actual project.
        .insert_resource(FrustumCulling(false))
        // Scale the camera aabb or it will be hard to see the chunk while disappearing/appearing.
        .insert_resource(CameraAabbScale(Vec2::splat(0.3)))
        .run();
}

#[derive(Event, Debug, Clone, Copy)]
struct GenerateChunk(IVec2);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // When the detect aabb is intersected with a invisible chunk,
    // all the chunks that are intercected with the update aabb must be visible.

    // Which means we need to first detect the chunks that are intersected with the detect aabb,
    // and if every one is visible, then do nothing else load/generate chunks that are intersected with the update aabb.
    commands.spawn((Camera2dBundle::default(), CameraChunkUpdater::new(1.3, 2.2)));

    let entity = commands.spawn_empty().id();
    let mut tilemap = TilemapBundle {
        name: TilemapName("laggy_map".to_string()),
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

    // Reserve means to tell the crate that this chunk is actually exists
    // but it's not on the tilemap yet.
    // So when the camera enters/leaves this chunk, you will receive the event
    // for this chunk.
    tilemap.storage.reserve_many(
        (-7..=6)
            .into_iter()
            .flat_map(move |x| (-7..=6).into_iter().map(move |y| IVec2 { x, y })),
    );

    // In this example, we will load from/save to the disk when the camera enters/leaves the chunk,
    // So if you are running this example for the first time, you need to uncomment the following lines,
    // and scan the whole tilemap until all the chunks are disappeared.
    // Then all the chunks will be saved to your disk.
    // Or you can write your own code to make allow the chunks to be generated at runtime.

    // tilemap.storage.fill_rect(
    //     &mut commands,
    //     bevy_entitiles::math::TileArea::new(IVec2 { x: -100, y: -100 }, UVec2 { x: 200, y: 200 }),
    //     TileBuilder::new().with_layer(0, TileLayer::new().with_texture_index(0)),
    // );

    #[allow(unused_mut)]
    let mut physics_tilemap = PhysicsTilemap::new_with_chunk_size(16);
    // physics_tilemap.fill_rect_custom(
    //     bevy_entitiles::math::TileArea::new(IVec2 { x: -100, y: -100 }, UVec2 { x: 200, y: 200 }),
    //     |_| {
    //         if rand::random::<u32>() % 10 == 0 {
    //             Some(PhysicsTile {
    //                 rigid_body: true,
    //                 friction: Some(0.2),
    //             })
    //         } else {
    //             None
    //         }
    //     },
    //     false,
    // );
    commands.entity(entity).insert(physics_tilemap);

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

    // You can actually do everything you want
    // This case we load/save the chunk

    if !to_load.is_empty() {
        load_cache.schedule_many(
            &mut commands,
            tilemap,
            TilemapLayer::all(),
            to_load.into_iter(),
        );
    }

    if !to_unload.is_empty() {
        save_cache.schedule_many(
            &mut commands,
            tilemap,
            TilemapLayer::all(),
            to_unload.into_iter(),
        );
    }
}
