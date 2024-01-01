use bevy::{
    app::{App, Startup},
    core_pipeline::core_2d::Camera2dBundle,
    ecs::system::{Commands, ResMut},
    DefaultPlugins,
};
use bevy_entitiles::{EntiTilesPlugin, ldtk::resources::LdtkLevelManager};
use helpers::EntiTilesDebugPlugin;

mod helpers;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EntiTilesPlugin, EntiTilesDebugPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, mut manager: ResMut<LdtkLevelManager>) {
    commands.spawn(Camera2dBundle::default());
}
