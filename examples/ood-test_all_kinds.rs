use bevy::{
    app::{App, Startup},
    asset::AssetServer,
    core_pipeline::core_2d::Camera2dBundle,
    ecs::{
        entity::Entity,
        system::{Commands, Res},
    },
    math::{IVec2, UVec2, Vec2},
    render::{color::Color, render_resource::FilterMode},
    DefaultPlugins,
};
use bevy_entitiles::{
    math::TileArea,
    tilemap::{
        bundles::StandardTilemapBundle,
        map::{
            TilePivot, TileRenderSize, TilemapAxisFlip, TilemapRotation, TilemapSlotSize, TilemapStorage, TilemapTexture, TilemapTextureDescriptor, TilemapTransform, TilemapType
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

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    for (ty, size, count, tex_path, offset, flip) in [
        // (
        //     TilemapType::Square,
        //     UVec2::new(16, 16),
        //     UVec2::new(2, 2),
        //     "test_square.png",
        //     Vec2::ZERO,
        //     TilemapAxisFlip::X,
        // ),
        (
            TilemapType::Isometric,
            UVec2::new(32, 16),
            UVec2::new(1, 2),
            "test_isometric.png",
            Vec2::new(300., 0.),
            TilemapAxisFlip::X,
        ),
        (
            TilemapType::Hexagonal(40),
            UVec2::new(28, 56),
            UVec2::new(2, 1),
            "test_hexagonal.png",
            Vec2::new(0., 0.),
            TilemapAxisFlip::X,
        ),
    ] {
        let entity = commands.spawn_empty().id();
        let mut storage = TilemapStorage::new(4, entity);
        storage.fill_rect_custom(
            &mut commands,
            TileArea::new(IVec2::new(0, 0), UVec2::new(4, 4)),
            |index| {
                if index.x == 0 && index.y == 0 {
                    Some(
                        TileBuilder::new()
                            .with_layer(0, TileLayer::new().with_texture_index(0))
                            .with_color(Color::BLUE.into()),
                    )
                } else if index.x == 0 || index.y == 0 {
                    Some(
                        TileBuilder::new()
                            .with_layer(0, TileLayer::new().with_texture_index(0))
                            .with_color(if index.y == 0 {
                                Color::RED.into()
                            } else {
                                Color::GREEN.into()
                            }),
                    )
                } else {
                    Some(TileBuilder::new().with_layer(0, TileLayer::new().with_texture_index(0)))
                }
            },
            false,
        );

        commands.entity(entity).insert(StandardTilemapBundle {
            storage: storage.clone(),
            ty,
            tile_render_size: TileRenderSize(size.as_vec2()),
            slot_size: TilemapSlotSize(size.as_vec2()),
            texture: TilemapTexture::new(
                asset_server.load(tex_path),
                TilemapTextureDescriptor::new(size * count, size, FilterMode::Nearest),
                TilemapRotation::None,
            ),
            transform: TilemapTransform::from_translation(offset),
            axis_flip: flip,
            // tile_pivot: TilePivot(Vec2::new(0.8, 0.2)),
            ..Default::default()
        });
    }
}
