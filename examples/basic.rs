use bevy::{
    app::PluginGroup,
    math::{IVec2, Vec4},
    prelude::{App, AssetServer, Camera2dBundle, Commands, Res, Startup, UVec2, Vec2},
    render::{color::Color, render_resource::FilterMode},
    window::{PresentMode, Window, WindowPlugin},
    DefaultPlugins,
};
use bevy_entitiles::{
    debug::EntiTilesDebugPlugin,
    math::TileArea,
    render::texture::{TilemapTexture, TilemapTextureDescriptor},
    tilemap::{
        layer::{LayerUpdater, TileLayer, TileLayerPosition, TileUpdater},
        map::{TilemapBuilder, TilemapRotation},
        tile::{TileBuilder, TileFlip, TileType},
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
            EntiTilesHelpersPlugin,
            EntiTilesDebugPlugin,
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, assets_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let mut tilemap = TilemapBuilder::new(
        TileType::Square,
        Vec2 { x: 16., y: 16. },
        "test_map".to_string(),
    )
    .with_texture(TilemapTexture::new(
        assets_server.load("test_square.png"),
        TilemapTextureDescriptor::new(
            UVec2 { x: 32, y: 32 },
            UVec2 { x: 16, y: 16 },
            FilterMode::Nearest,
        ),
        TilemapRotation::None,
    ))
    .build(&mut commands);

    tilemap.fill_rect(
        &mut commands,
        TileArea::new(IVec2::ZERO, UVec2 { x: 20, y: 10 }),
        TileBuilder::new().with_layer(0, TileLayer::new().with_texture_index(0)),
    );

    tilemap.fill_rect(
        &mut commands,
        TileArea::new(IVec2 { x: 2, y: 2 }, UVec2 { x: 10, y: 7 }),
        TileBuilder::new()
            .with_layer(0, TileLayer::new().with_texture_index(1))
            .with_color(Vec4::new(0.8, 1., 0.8, 0.5)),
    );

    tilemap.set(
        &mut commands,
        IVec2 { x: 18, y: 8 },
        TileBuilder::new()
            .with_layer(0, TileLayer::new().with_texture_index(0))
            .with_color(Color::BLUE.into()),
    );

    tilemap.set(
        &mut commands,
        IVec2 { x: 1, y: 1 },
        TileBuilder::new().with_layer(
            1,
            TileLayer::new()
                .with_texture_index(1)
                .with_flip(TileFlip::Horizontal),
        ),
    );

    tilemap.set(
        &mut commands,
        IVec2 { x: 1, y: 2 },
        TileBuilder::new().with_layer(
            0,
            TileLayer::new()
                .with_texture_index(1)
                .with_flip(TileFlip::Vertical),
        ),
    );

    tilemap.set(
        &mut commands,
        IVec2 { x: 1, y: 3 },
        TileBuilder::new().with_layer(
            0,
            TileLayer::new()
                .with_texture_index(1)
                .with_flip(TileFlip::Both),
        ),
    );

    tilemap.update_rect(
        &mut commands,
        TileArea::new(IVec2 { x: 1, y: 3 }, UVec2 { x: 3, y: 3 }),
        TileUpdater {
            layer: Some(LayerUpdater {
                position: TileLayerPosition::Index(1),
                layer: TileLayer::new().with_texture_index(3),
            }),
            ..Default::default()
        },
    );

    commands.entity(tilemap.id()).insert(tilemap);

    let mut tilemap = TilemapBuilder::new(
        TileType::Isometric,
        Vec2 { x: 32., y: 16. },
        "test_map".to_string(),
    )
    .with_texture(TilemapTexture::new(
        assets_server.load("test_isometric.png"),
        TilemapTextureDescriptor::new(
            UVec2 { x: 32, y: 32 },
            UVec2 { x: 32, y: 16 },
            FilterMode::Nearest,
        ),
        TilemapRotation::None,
    ))
    .with_translation(Vec2 { x: -400., y: 0. })
    .build(&mut commands);

    tilemap.fill_rect(
        &mut commands,
        TileArea::new(IVec2::ZERO, UVec2 { x: 20, y: 10 }),
        TileBuilder::new().with_layer(0, TileLayer::new().with_texture_index(0)),
    );

    commands.entity(tilemap.id()).insert(tilemap);

    let mut tilemap = TilemapBuilder::new(
        TileType::Square,
        Vec2 { x: 16., y: 16. },
        "test_map".to_string(),
    )
    .with_translation(Vec2 { x: 0., y: -300. })
    .build(&mut commands);

    tilemap.fill_rect(
        &mut commands,
        TileArea::new(IVec2::ZERO, UVec2 { x: 20, y: 10 }),
        TileBuilder::new()
            .with_layer(0, TileLayer::new().with_texture_index(0))
            .with_color(Vec4::new(1., 1., 0., 1.)),
    );

    commands.entity(tilemap.id()).insert(tilemap);
}
