use bevy::{
    app::{App, PluginGroup, Startup, Update},
    asset::{AssetServer, Assets},
    core_pipeline::core_2d::Camera2dBundle,
    ecs::{
        entity::Entity,
        system::{Commands, Query, Res, ResMut},
    },
    math::{IVec2, UVec2, Vec2, Vec4},
    render::{
        color::Color,
        render_resource::FilterMode,
        texture::{Image, ImagePlugin},
    },
    sprite::{Sprite, SpriteBundle},
    window::{PresentMode, Window, WindowPlugin},
    DefaultPlugins,
};
use bevy_entitiles::{
    math::TileArea,
    render::{
        bake::{BakedTilemap, TilemapBaker},
        material::StandardTilemapMaterial,
    },
    tilemap::{
        bundles::StandardTilemapBundle,
        map::{
            TileRenderSize, TilemapLayerOpacities, TilemapRotation, TilemapSlotSize,
            TilemapStorage, TilemapTexture, TilemapTextureDescriptor, TilemapTextures, TilemapType,
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
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        present_mode: PresentMode::Immediate,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .set(ImagePlugin::default_nearest()),
            EntiTilesPlugin,
            EntiTilesHelpersPlugin { inspector: true },
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, fetch_bake_result)
        .run();
}

fn setup(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardTilemapMaterial>>,
    mut textures: ResMut<Assets<TilemapTextures>>,
) {
    commands.spawn(Camera2dBundle::default());

    let entity = commands.spawn_empty().id();
    let mut tilemap = StandardTilemapBundle {
        tile_render_size: TileRenderSize(Vec2::splat(16.)),
        slot_size: TilemapSlotSize(Vec2::splat(16.)),
        ty: TilemapType::Square,
        storage: TilemapStorage::new(32, entity),
        material: materials.add(StandardTilemapMaterial::default()),
        textures: textures.add(TilemapTexture::new(
            asset_server.load("test_square.png"),
            TilemapTextureDescriptor::new(UVec2::splat(32), UVec2::splat(16), FilterMode::Nearest),
            TilemapRotation::None,
        )),
        layer_opacities: TilemapLayerOpacities(Vec4::new(0.8, 0.5, 0.1, 0.3)),
        ..Default::default()
    };

    tilemap.storage.fill_rect(
        &mut commands,
        TileArea::from_min_max(IVec2::ZERO, IVec2::splat(4)),
        TileBuilder::new().with_layer(0, TileLayer::no_flip(0)),
    );

    tilemap.storage.fill_rect(
        &mut commands,
        TileArea::from_min_max(IVec2::new(5, 0), IVec2::new(9, 4)),
        TileBuilder::new()
            .with_layer(0, TileLayer::no_flip(0))
            .with_layer(1, TileLayer::flip_h(1))
            .with_tint(Color::rgba_u8(68, 62, 185, 64)),
    );

    tilemap.storage.fill_rect(
        &mut commands,
        TileArea::new(IVec2::new(10, 0), UVec2::splat(5)),
        TileBuilder::new()
            .with_layer(0, TileLayer::no_flip(0))
            .with_layer(1, TileLayer::no_flip(1))
            .with_layer(2, TileLayer::no_flip(2))
            .with_layer(3, TileLayer::no_flip(3))
            .with_tint(Color::ORANGE_RED),
    );

    tilemap.storage.fill_rect(
        &mut commands,
        TileArea::new(IVec2::new(0, 10), UVec2::splat(10)),
        TileBuilder::new().with_layer(0, TileLayer::no_flip(2)),
    );

    commands.entity(entity).insert((
        tilemap,
        TilemapBaker {
            remove_after_done: true,
        },
    ));
}

fn fetch_bake_result(
    mut commands: Commands,
    mut baked_query: Query<(Entity, &mut BakedTilemap)>,
    mut images: ResMut<Assets<Image>>,
) {
    let Ok((tilemap, mut baked)) = baked_query.get_single_mut() else {
        return;
    };

    commands.spawn(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(baked.size_px.as_vec2()),
            ..Default::default()
        },
        texture: images.add(baked.texture.take().unwrap()),
        ..Default::default()
    });

    commands.entity(tilemap).despawn();
}
