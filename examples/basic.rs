use bevy::{
    app::{PluginGroup, Update},
    asset::Assets,
    ecs::{
        query::With,
        system::{Query, ResMut},
    },
    input::{keyboard::KeyCode, ButtonInput},
    math::IVec2,
    prelude::{App, AssetServer, Camera2dBundle, Commands, Res, Startup, UVec2, Vec2},
    render::{color::Color, render_resource::FilterMode, view::Visibility},
    window::{PresentMode, Window, WindowPlugin},
    DefaultPlugins,
};
use bevy_entitiles::{
    math::TileArea,
    render::material::StandardTilemapMaterial,
    tilemap::{
        bundles::{StandardPureColorTilemapBundle, StandardTilemapBundle},
        map::{
            TileRenderSize, TilemapName, TilemapRotation, TilemapSlotSize, TilemapStorage,
            TilemapTexture, TilemapTextureDescriptor, TilemapTransform, TilemapType,
        },
        tile::{LayerUpdater, TileBuilder, TileLayer, TileLayerPosition, TileUpdater},
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
            EntiTilesHelpersPlugin::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, toggle)
        .run();
}

fn setup(
    mut commands: Commands,
    assets_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardTilemapMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());

    let entity = commands.spawn_empty().id();
    // You can go to each component's definition to see what they do.
    let mut tilemap = StandardTilemapBundle {
        name: TilemapName("test_map".to_string()),
        tile_render_size: TileRenderSize(Vec2 { x: 16., y: 16. }),
        slot_size: TilemapSlotSize(Vec2 { x: 16., y: 16. }),
        ty: TilemapType::Square,
        storage: TilemapStorage::new(16, entity),
        material: materials.add(StandardTilemapMaterial::default()),
        texture: TilemapTexture::new(
            assets_server.load("test_square.png"),
            TilemapTextureDescriptor::new(
                UVec2 { x: 32, y: 32 },
                UVec2 { x: 16, y: 16 },
                FilterMode::Nearest,
            ),
            TilemapRotation::None,
        ),
        ..Default::default()
    };

    tilemap.storage.fill_rect(
        &mut commands,
        TileArea::new(IVec2::ZERO, UVec2 { x: 20, y: 10 }),
        TileBuilder::new().with_layer(0, TileLayer::no_flip(0)),
    );

    tilemap.storage.fill_rect(
        &mut commands,
        TileArea::new(IVec2 { x: 2, y: 2 }, UVec2 { x: 10, y: 7 }),
        TileBuilder::new()
            .with_layer(0, TileLayer::no_flip(1))
            .with_tint(Color::rgba(0.8, 1., 0.8, 0.5)),
    );

    tilemap.storage.set(
        &mut commands,
        IVec2 { x: 18, y: 8 },
        TileBuilder::new()
            .with_layer(0, TileLayer::no_flip(0))
            .with_tint(Color::BLUE),
    );

    tilemap.storage.set(
        &mut commands,
        IVec2 { x: 1, y: 1 },
        TileBuilder::new().with_layer(1, TileLayer::flip_h(1)),
    );

    tilemap.storage.set(
        &mut commands,
        IVec2 { x: 1, y: 2 },
        TileBuilder::new().with_layer(0, TileLayer::flip_v(1)),
    );

    tilemap.storage.set(
        &mut commands,
        IVec2 { x: 1, y: 3 },
        TileBuilder::new().with_layer(0, TileLayer::flip_both(1)),
    );

    tilemap.storage.update_rect(
        &mut commands,
        TileArea::new(IVec2 { x: 1, y: 3 }, UVec2 { x: 3, y: 3 }),
        TileUpdater {
            layer: Some(LayerUpdater {
                position: TileLayerPosition::Top,
                layer: TileLayer::no_flip(3),
            }),
            ..Default::default()
        },
    );

    commands.entity(entity).insert(tilemap);

    let entity = commands.spawn_empty().id();
    let mut tilemap = StandardTilemapBundle {
        name: TilemapName("test_map".to_string()),
        tile_render_size: TileRenderSize(Vec2 { x: 32., y: 16. }),
        slot_size: TilemapSlotSize(Vec2 { x: 32., y: 16. }),
        ty: TilemapType::Isometric,
        storage: TilemapStorage::new(32, entity),
        material: materials.add(StandardTilemapMaterial {
            tint: Color::TOMATO,
        }),
        texture: TilemapTexture::new(
            assets_server.load("test_isometric.png"),
            TilemapTextureDescriptor::new(
                UVec2 { x: 32, y: 32 },
                UVec2 { x: 32, y: 16 },
                FilterMode::Nearest,
            ),
            TilemapRotation::None,
        ),
        transform: TilemapTransform {
            translation: Vec2 { x: -400., y: 0. },
            ..Default::default()
        },
        ..Default::default()
    };

    tilemap.storage.fill_rect(
        &mut commands,
        TileArea::new(IVec2::ZERO, UVec2 { x: 20, y: 10 }),
        TileBuilder::new().with_layer(0, TileLayer::no_flip(0)),
    );

    commands.entity(entity).insert(tilemap);

    let entity = commands.spawn_empty().id();
    let mut tilemap = StandardPureColorTilemapBundle {
        name: TilemapName("test_map".to_string()),
        tile_render_size: TileRenderSize(Vec2 { x: 16., y: 16. }),
        slot_size: TilemapSlotSize(Vec2 { x: 16., y: 16. }),
        ty: TilemapType::Square,
        storage: TilemapStorage::new(32, entity),
        material: materials.add(StandardTilemapMaterial::default()),
        transform: TilemapTransform {
            translation: Vec2 { x: 0., y: -300. },
            ..Default::default()
        },
        ..Default::default()
    };

    tilemap.storage.fill_rect(
        &mut commands,
        TileArea::new(IVec2::ZERO, UVec2 { x: 20, y: 10 }),
        TileBuilder::new()
            .with_layer(0, TileLayer::no_flip(0))
            .with_tint(Color::rgba(1., 1., 0., 1.)),
    );

    commands.entity(entity).insert(tilemap);
}

fn toggle(
    mut tilemaps_query: Query<&mut Visibility, With<TilemapStorage>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if input.just_pressed(KeyCode::Space) {
        for mut visibility in tilemaps_query.iter_mut() {
            *visibility = match *visibility {
                Visibility::Inherited => Visibility::Hidden,
                Visibility::Hidden => Visibility::Visible,
                Visibility::Visible => Visibility::Hidden,
            }
        }
    }
}
