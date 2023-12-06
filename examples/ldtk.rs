use bevy::{
    app::{App, Startup},
    core_pipeline::core_2d::Camera2dBundle,
    ecs::system::Commands,
    DefaultPlugins,
};
use bevy_entitiles::{serializing::ldtk::LdtkLoader, EntiTilesPlugin};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EntiTilesPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn(LdtkLoader {
        path: "assets/ldtk/test_ldtk.json".to_string(),
    });
}
