use bevy::{color::palettes::css::ORANGE_RED, prelude::*, window::PresentMode};
use bevy_entitiles::prelude::*;
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
        layer_opacities: TilemapLayerOpacities(Vec4::new(0.8, 0.5, 0.1, 0.3)),
        ..Default::default()
    };

    tilemap.storage.fill_rect(
        &mut commands,
        GridRect::from_min_max(IVec2::ZERO, IVec2::splat(4)),
        TileBuilder::new().with_layer(0, TileLayer::no_flip_at(0, 0)),
    );

    tilemap.storage.fill_rect(
        &mut commands,
        GridRect::from_min_max(IVec2::new(5, 0), IVec2::new(9, 4)),
        TileBuilder::new()
            .with_layer(0, TileLayer::no_flip_at(0, 0))
            .with_layer(1, TileLayer::flip_h_at(0, 1))
            .with_tint(LinearRgba::new(
                68. / 255.,
                62. / 255.,
                185. / 255.,
                64. / 255.,
            )),
    );

    tilemap.storage.fill_rect(
        &mut commands,
        GridRect::new(IVec2::new(10, 0), UVec2::splat(5)),
        TileBuilder::new()
            .with_layer(0, TileLayer::no_flip_at(0, 0))
            .with_layer(1, TileLayer::no_flip_at(0, 1))
            .with_layer(2, TileLayer::no_flip_at(0, 2))
            .with_layer(3, TileLayer::no_flip_at(0, 3))
            .with_tint(ORANGE_RED.into()),
    );

    tilemap.storage.fill_rect(
        &mut commands,
        GridRect::new(IVec2::new(0, 10), UVec2::splat(10)),
        TileBuilder::new().with_layer(0, TileLayer::no_flip_at(1, 2)),
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
