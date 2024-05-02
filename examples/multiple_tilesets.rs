use bevy::{
    app::{App, Startup},
    asset::{AssetServer, Assets},
    core_pipeline::core_2d::Camera2dBundle,
    ecs::system::{Commands, Res, ResMut},
    math::{IVec2, UVec2, Vec2},
    render::render_resource::FilterMode,
    DefaultPlugins,
};
use bevy_entitiles::{
    math::TileArea,
    render::material::StandardTilemapMaterial,
    tilemap::{
        bundles::StandardTilemapBundle,
        map::{
            TileRenderSize, TilemapSlotSize, TilemapStorage, TilemapTexture,
            TilemapTextureDescriptor, TilemapTextures,
        },
        tile::{TileBuilder, TileLayer},
    },
    EntiTilesPlugin, DEFAULT_CHUNK_SIZE,
};
use helpers::EntiTilesHelpersPlugin;

mod helpers;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            EntiTilesPlugin,
            EntiTilesHelpersPlugin::default(),
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardTilemapMaterial>>,
    mut textures: ResMut<Assets<TilemapTextures>>,
) {
    commands.spawn(Camera2dBundle::default());

    let entity = commands.spawn_empty().id();
    let mut tilemap = StandardTilemapBundle {
        tile_render_size: TileRenderSize(Vec2::splat(16.)),
        slot_size: TilemapSlotSize(Vec2::splat(16.)),
        storage: TilemapStorage::new(DEFAULT_CHUNK_SIZE, entity),
        material: materials.add(StandardTilemapMaterial::default()),
        textures: textures.add(TilemapTextures::new(
            vec![
                TilemapTexture::new(
                    asset_server.load("test_square.png"),
                    TilemapTextureDescriptor::new(UVec2::splat(32), UVec2::splat(16)),
                ),
                TilemapTexture::new(
                    asset_server.load("test_wfc.png"),
                    TilemapTextureDescriptor::new(UVec2 { x: 48, y: 32 }, UVec2 { x: 16, y: 16 }),
                ),
            ],
            FilterMode::Nearest,
        )),
        ..Default::default()
    };

    // Without `atlas` feature:
    tilemap.storage.fill_rect(
        &mut commands,
        TileArea::new(IVec2::ZERO, UVec2::splat(4)),
        TileBuilder::new().with_layer(0, TileLayer::no_flip(0)),
    );

    tilemap.storage.fill_rect(
        &mut commands,
        TileArea::new(IVec2::new(5, 0), UVec2::splat(4)),
        TileBuilder::new().with_layer(0, TileLayer::no_flip(4)),
    );

    // With `atlas` feature:
    // tilemap.storage.fill_rect(
    //     &mut commands,
    //     TileArea::new(IVec2::ZERO, UVec2::splat(4)),
    //     TileBuilder::new().with_layer(0, TileLayer::no_flip(0, 0)),
    // );

    // tilemap.storage.fill_rect(
    //     &mut commands,
    //     TileArea::new(IVec2::new(5, 0), UVec2::splat(4)),
    //     TileBuilder::new().with_layer(0, TileLayer::no_flip(1, 0)),
    // );

    commands.entity(entity).insert(tilemap);
}
