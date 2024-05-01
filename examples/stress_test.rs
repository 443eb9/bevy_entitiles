use bevy::{
    app::{App, PluginGroup, Startup},
    asset::{AssetServer, Assets},
    core_pipeline::core_2d::Camera2dBundle,
    ecs::system::{Commands, Res, ResMut},
    math::{IVec2, UVec2, Vec2},
    render::render_resource::FilterMode,
    window::{PresentMode, Window, WindowPlugin},
    DefaultPlugins,
};
use bevy_entitiles::{
    math::TileArea,
    render::{cull::FrustumCulling, material::StandardTilemapMaterial},
    tilemap::{
        bundles::StandardTilemapBundle,
        map::{
            TileRenderSize, TilemapRotation, TilemapSlotSize, TilemapStorage, TilemapTexture,
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
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    present_mode: PresentMode::Immediate,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            EntiTilesPlugin,
            EntiTilesHelpersPlugin { inspector: false },
        ))
        .add_systems(Startup, setup)
        .insert_resource(FrustumCulling(false))
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
        tile_render_size: TileRenderSize(Vec2::new(16., 16.)),
        slot_size: TilemapSlotSize(Vec2::new(16., 16.)),
        ty: TilemapType::Square,
        storage: TilemapStorage::new(32, entity),
        material: materials.add(StandardTilemapMaterial::default()),
        textures: textures.add(TilemapTextures::single(
            TilemapTexture::new(
                asset_server.load("test_square.png"),
                TilemapTextureDescriptor::new(UVec2::splat(32), UVec2::splat(16)),
                TilemapRotation::None,
            ),
            FilterMode::Nearest,
        )),
        ..Default::default()
    };

    tilemap.storage.fill_rect(
        &mut commands,
        TileArea::new(IVec2::splat(-500), UVec2::splat(1000)),
        TileBuilder::new().with_layer(0, TileLayer::no_flip(0)),
    );

    commands.entity(entity).insert(tilemap);
}
