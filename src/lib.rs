use bevy::{app::Update, ecs::schedule::States, prelude::Plugin};
use render::{texture, EntiTilesRendererPlugin};

#[cfg(feature = "algorithm")]
pub mod algorithm;
#[cfg(feature = "debug")]
pub mod debug;
pub mod math;
pub mod render;
#[cfg(feature = "serializing")]
pub mod serializing;
pub mod tilemap;
pub mod ui;

pub struct EntiTilesPlugin;

impl Plugin for EntiTilesPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Update, texture::set_texture_usage);

        app.add_state::<EntiTilesStates>();

        app.add_plugins(EntiTilesRendererPlugin);

        #[cfg(feature = "algorithm")]
        app.add_plugins(algorithm::EntiTilesAlgorithmPlugin);
        #[cfg(any(feature = "physics_rapier", feature = "physics_xpbd"))]
        app.add_plugins(tilemap::physics::EntiTilesPhysicsPlugin);
        #[cfg(feature = "serializing")]
        app.add_plugins(serializing::EntiTilesSerializingPlugin);
        #[cfg(feature = "ui")]
        app.add_plugins(ui::EntiTilesUiPlugin);
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
pub enum EntiTilesStates {
    #[default]
    Simulating,
    #[cfg(feature = "editor")]
    Editing,
}
