use bevy::{
    app::{App, Startup},
    asset::AssetServer,
    core_pipeline::core_2d::Camera2dBundle,
    ecs::system::{Commands, Res},
    math::{IVec2, UVec2, Vec2},
    render::render_resource::FilterMode,
    DefaultPlugins,
};
use bevy_entitiles::{
    math::TileArea,
    render::{
        buffer::TileAnimation,
        texture::{TilemapTexture, TilemapTextureDescriptor},
    },
    tilemap::{
        map::{TilemapBuilder, TilemapRotation},
        tile::{TileBuilder, TilemapType},
    },
    EntiTilesPlugin,
};
use helpers::EntiTilesHelpersPlugin;

mod helpers;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EntiTilesPlugin, EntiTilesHelpersPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let mut tilemap = TilemapBuilder::new(
        TilemapType::Square,
        Vec2 { x: 16., y: 16. },
        "test_map".to_string(),
    )
    .with_texture(TilemapTexture::new(
        asset_server.load("test_square.png"),
        TilemapTextureDescriptor::new(
            UVec2 { x: 32, y: 32 },
            UVec2 { x: 16, y: 16 },
            FilterMode::Nearest,
        ),
        TilemapRotation::None,
    ))
    .build(&mut commands);

    let anim_a = tilemap.register_animation(TileAnimation::new(vec![0, 1, 2, 3], 2.));
    let anim_b = tilemap.register_animation(TileAnimation::new(vec![0, 1, 2], 3.));

    tilemap.fill_rect(
        &mut commands,
        TileArea::new(IVec2::ZERO, UVec2 { x: 20, y: 20 }),
        TileBuilder::new().with_animation(anim_a),
    );

    tilemap.fill_rect(
        &mut commands,
        TileArea::new(IVec2::ZERO, UVec2 { x: 10, y: 10 }),
        TileBuilder::new().with_animation(anim_b),
    );

    commands.entity(tilemap.id()).insert(tilemap);
}
