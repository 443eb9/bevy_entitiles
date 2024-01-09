use bevy::{
    app::{App, Startup},
    ecs::system::Commands,
    DefaultPlugins, core_pipeline::core_2d::Camera2dBundle,
};
use bevy_entitiles::{debug::EntiTilesDebugPlugin, EntiTilesPlugin};
use helpers::EntiTilesHelpersPlugin;

mod helpers;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            EntiTilesPlugin,
            EntiTilesHelpersPlugin,
            EntiTilesDebugPlugin,
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
