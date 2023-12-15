use bevy::{
    asset::AssetServer,
    ecs::system::Res,
    prelude::{App, Camera2dBundle, Commands, Startup, UVec2, Vec2},
    render::render_resource::FilterMode,
    DefaultPlugins,
};
use bevy_entitiles::{
    algorithm::wfc::{AsyncWfcRunner, WfcRunner},
    math::FillArea,
    render::texture::{TilemapTexture, TilemapTextureDescriptor},
    tilemap::{map::TilemapBuilder, tile::TileType},
    EntiTilesPlugin,
};
use helpers::EntiTilesDebugPlugin;

mod helpers;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EntiTilesPlugin, EntiTilesDebugPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let (tilemap_entity, tilemap) = TilemapBuilder::new(
        TileType::Square,
        UVec2 { x: 50, y: 50 },
        Vec2 { x: 16., y: 16. },
        "test_map".to_string(),
    )
    .with_texture(TilemapTexture::new(
        asset_server.load("test_wfc.png"),
        TilemapTextureDescriptor::new(
            UVec2 { x: 48, y: 32 },
            UVec2 { x: 16, y: 16 },
            FilterMode::Nearest,
        ),
    ))
    .build(&mut commands);

    commands.entity(tilemap_entity).insert((
        WfcRunner::from_simple_config(
            &tilemap,
            "examples/wfc_config.ron".to_string(),
            FillArea::full(&tilemap),
            Some(0),
        )
        // use weights OR custom_sampler
        // .with_weights("examples/wfc_weights.ron".to_string())
        .with_retrace_settings(Some(8), Some(1000000))
        .with_fallback(Box::new(|_, e, _, _| {
            println!("Failed to generate: {:?}", e)
        })),
        AsyncWfcRunner,
    ));
}
