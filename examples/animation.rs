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
            TileRenderSize, TilemapRotation, TilemapSlotSize, TilemapStorage, TilemapTexture,
            TilemapTextureDescriptor, TilemapType,
        },
        tile::{RawTileAnimation, TileBuilder},
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
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardTilemapMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());

    let entity = commands.spawn_empty().id();
    let mut tilemap = StandardTilemapBundle {
        tile_render_size: TileRenderSize(Vec2::new(16., 16.)),
        slot_size: TilemapSlotSize(Vec2::new(16., 16.)),
        ty: TilemapType::Square,
        storage: TilemapStorage::new(16, entity),
        material: materials.add(StandardTilemapMaterial {
            texture: Some(TilemapTexture::new(
                asset_server.load("test_square.png"),
                TilemapTextureDescriptor::new(
                    UVec2 { x: 32, y: 32 },
                    UVec2 { x: 16, y: 16 },
                    FilterMode::Nearest,
                ),
                TilemapRotation::None,
            )),
            ..Default::default()
        }),
        ..Default::default()
    };

    let anim_a = tilemap.animations.register(RawTileAnimation {
        fps: 2,
        sequence: vec![0, 1, 2, 3],
    });
    let anim_b = tilemap.animations.register(RawTileAnimation {
        fps: 3,
        sequence: vec![0, 1, 2],
    });

    tilemap.storage.fill_rect(
        &mut commands,
        TileArea::new(IVec2::ZERO, UVec2 { x: 20, y: 20 }),
        TileBuilder::new().with_animation(anim_a),
    );

    tilemap.storage.fill_rect(
        &mut commands,
        TileArea::new(IVec2::ZERO, UVec2 { x: 10, y: 10 }),
        TileBuilder::new().with_animation(anim_b),
    );

    commands.entity(entity).insert(tilemap);
}
