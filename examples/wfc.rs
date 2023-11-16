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
use indexmap::IndexSet;

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
        UVec2 { x: 10, y: 10 },
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

    commands
        .entity(tilemap_entity)
        .insert(WfcRunner::from_config(
            "examples/wfc_config.ron".to_string(),
            None,
            // just a simple example, you can use some noise function
            Some(Box::new(|psbs: &IndexSet<u16>, index: UVec2| 0)),
            FillArea::full(&tilemap),
            None,
            None,
        ));
}
