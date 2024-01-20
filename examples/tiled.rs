use bevy::{
    app::{App, Startup},
    core_pipeline::core_2d::Camera2dBundle,
    ecs::system::Commands,
    DefaultPlugins,
};
use bevy_entitiles::{
    tiled::xml::{tileset::Tileset, TiledTilemap},
    EntiTilesPlugin,
};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EntiTilesPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    dbg!(quick_xml::de::from_str::<TiledTilemap>(
        std::fs::read_to_string("assets/tiled/tilemaps/orthogonal.tmx")
        // std::fs::read_to_string("assets/tiled/tilemaps/hexagonal.tmx")
        // std::fs::read_to_string("assets/tiled/tilemaps/isometric.tmx")
            .unwrap()
            .as_str(),
    )
    .unwrap());

    // dbg!(quick_xml::de::from_str::<Tileset>(
    //     std::fs::read_to_string("assets/tiled/tilesets/Tileset1.tsx")
    //         .unwrap()
    //         .as_str(),
    // )
    // .unwrap());
}
