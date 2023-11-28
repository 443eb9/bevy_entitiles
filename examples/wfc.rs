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
    render::texture::TilemapTextureDescriptor,
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
        Vec2 { x: 32., y: 32. },
    )
    .with_texture(
        asset_server.load("test_wfc.png"),
        TilemapTextureDescriptor::from_full_grid(
            UVec2 { x: 48, y: 32 },
            UVec2 { x: 3, y: 2 },
            FilterMode::Nearest,
        ),
    )
    .build(&mut commands);

    // The following code is NOT suitable for every project.
    // I just want to show you the full functionality.
    // Please adjust them before you run the program.
    commands.entity(tilemap_entity).insert((
        WfcRunner::from_config(
            "examples/wfc_config.ron".to_string(),
            FillArea::full(&tilemap),
            Some(0),
        )
        // just a simple example, you can use some noise function
        // .with_custom_sampler(Box::new(|tile, rng| {
        //     let psbs = tile.get_psbs_vec();
        //     psbs[rng.sample(Uniform::new(0, psbs.len()))]
        // }))
        // use weights OR custom_sampler
        // .with_weights("examples/wfc_weights.ron".to_string())
        .with_retrace_settings(Some(8), Some(1000000))
        .with_fallback(Box::new(|_, e, _, _| {
            println!("Failed to generate: {:?}", e)
        })),
        AsyncWfcRunner,
    ));
}
