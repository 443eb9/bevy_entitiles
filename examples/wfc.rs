use bevy::{
    asset::AssetServer,
    ecs::system::Res,
    prelude::{App, Camera2dBundle, Commands, Startup, UVec2, Update, Vec2},
    render::render_resource::FilterMode,
    DefaultPlugins,
};
use bevy_entitiles::{
    algorithm::wfc::WfcRunner,
    debug::camera_movement::camera_control,
    math::FillArea,
    render::texture::TilemapTextureDescriptor,
    tilemap::{TileType, TilemapBuilder},
    EntiTilesPlugin,
};
use rand::Rng;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EntiTilesPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, camera_control)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let (tilemap_entity, tilemap) = TilemapBuilder::new(
        TileType::Square,
        UVec2 { x: 20, y: 20 },
        Vec2 { x: 32., y: 32. },
    )
    .with_texture(
        asset_server.load("test/test_wfc.png"),
        TilemapTextureDescriptor {
            tile_count: UVec2 { x: 3, y: 2 },
            tile_size: UVec2 { x: 16, y: 16 },
            filter_mode: FilterMode::Nearest,
        },
    )
    .build(&mut commands);

    // The following code is NOT reasonable for every project.
    // I just want to show you the full functionality.
    // Please adjust them before you run the program.
    commands.entity(tilemap_entity).insert(
        WfcRunner::from_config(
            "examples/wfc_config.ron".to_string(),
            None,
            // If you want to make custom_sampler work, replace the None with the commented code below.
            None,
            // just a simple example, you can use some noise function
            // Some(Box::new(|tile| {
            //     let psbs = tile.get_vec_psbs();
            //     psbs[rand::thread_rng().sample(rand::distributions::Uniform::from(0..psbs.len()))]
            // })),
            FillArea::full(&tilemap),
            None,
            None,
        )
        .with_retrace_settings(8, 10000)
        .with_fallback(Box::new(|e| println!("Failed to generate: {:?}", e))),
    );
}
