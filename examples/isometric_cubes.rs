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
    render::{chunk::RenderChunkSort, material::StandardTilemapMaterial},
    tilemap::{
        bundles::StandardTilemapBundle,
        map::{
            TileRenderSize, TilemapSlotSize, TilemapStorage, TilemapTexture,
            TilemapTextureDescriptor, TilemapTextures, TilemapType,
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
            DefaultPlugins,
            EntiTilesPlugin,
            EntiTilesHelpersPlugin::default(),
        ))
        .add_systems(Startup, setup)
        .insert_resource(RenderChunkSort::XReverseAndYReverse)
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
        tile_render_size: TileRenderSize(Vec2::splat(32.)),
        slot_size: TilemapSlotSize(Vec2::new(32., 16.)),
        ty: TilemapType::Isometric,
        // We can't take advantage of chunking for 3d isometric tilemaps.
        // Stacking tiles needs to do z test, and requires a depth texture for the pipeline.
        // But bevy doesn't support adding depth attachments to the pass in RenderCommand
        storage: TilemapStorage::new(1, entity),
        material: materials.add(StandardTilemapMaterial::default()),
        textures: textures.add(TilemapTextures::single(
            TilemapTexture::new(
                asset_server.load("test_isometric_cubes.png"),
                TilemapTextureDescriptor::new(UVec2::new(64, 32), UVec2::splat(32)),
            ),
            FilterMode::Nearest,
        )),
        ..Default::default()
    };

    tilemap.storage.fill_rect_custom(
        &mut commands,
        TileArea::new(IVec2::ZERO, UVec2::splat(8)),
        |index| Some(TileBuilder::new().with_layer(0, TileLayer::no_flip(index.x % 2))),
        false,
    );

    commands.entity(entity).insert(tilemap);
}
