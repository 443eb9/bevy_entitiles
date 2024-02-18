use bevy::{
    app::{App, Startup},
    asset::AssetServer,
    core_pipeline::core_2d::Camera2dBundle,
    ecs::system::{Commands, Res},
    math::{IVec2, UVec2, Vec2},
    render::render_resource::FilterMode,
    utils::HashMap,
    DefaultPlugins,
};
use bevy_entitiles::{
    math::TileArea,
    tilemap::{
        bundles::StandardTilemapBundle,
        coordinates::{self, StaggerMode},
        map::{
            TileRenderSize, TilemapRotation, TilemapSlotSize, TilemapStorage, TilemapTexture,
            TilemapTextureDescriptor, TilemapType,
        },
        physics::{DataPhysicsTilemap, PhysicsTile},
        tile::{TileBuilder, TileLayer},
    },
    EntiTilesPlugin, DEFAULT_CHUNK_SIZE,
};
use bevy_xpbd_2d::plugins::{PhysicsDebugPlugin, PhysicsPlugins};
use helpers::EntiTilesHelpersPlugin;

mod helpers;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            EntiTilesPlugin,
            EntiTilesHelpersPlugin::default(),
            PhysicsPlugins::default(),
            PhysicsDebugPlugin::default(),
        ))
        .add_systems(Startup, setup)
        // .add_systems(PostUpdate, save_tilemap)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let entity = commands.spawn_empty().id();
    let mut tilemap = StandardTilemapBundle {
        tile_render_size: TileRenderSize(Vec2::splat(16.)),
        slot_size: TilemapSlotSize(Vec2::splat(16.)),
        ty: TilemapType::Square,
        storage: TilemapStorage::new(DEFAULT_CHUNK_SIZE, entity),
        texture: TilemapTexture::new(
            asset_server.load("test_sqaure.png"),
            TilemapTextureDescriptor::new(UVec2::splat(32), UVec2::splat(16), FilterMode::Nearest),
            TilemapRotation::None,
        ),
        ..Default::default()
    };
    tilemap.storage.fill_rect(
        &mut commands,
        TileArea::new(IVec2::ZERO, UVec2::splat(5)),
        TileBuilder::new().with_layer(0, TileLayer::new().with_texture_index(0)),
    );

    let physics_tilemap = DataPhysicsTilemap::new(
        IVec2::ZERO,
        vec![
            1, 1, 1, 1, 1, //
            1, 0, 0, 0, 1, //
            1, 0, 0, 0, 1, //
            1, 0, 0, 0, 1, //
            1, 1, 1, 1, 1, //
        ],
        UVec2::splat(5),
        0,
        HashMap::from([(
            1,
            PhysicsTile {
                rigid_body: true,
                friction: Some(0.9),
            },
        )]),
    );

    commands.entity(entity).insert((tilemap, physics_tilemap));
}
