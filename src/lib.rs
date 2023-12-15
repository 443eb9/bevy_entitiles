use bevy::{app::Update, ecs::schedule::States, prelude::Plugin};
use render::{texture, EntiTilesRendererPlugin};
use tilemap::EntiTilesTilemapPlugin;

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

pub const MAX_TILESET_COUNT: usize = 4;
pub const MAX_LAYER_COUNT: usize = 4;
pub const MAX_ATLAS_COUNT: usize = 512;
pub const MAX_ANIM_COUNT: usize = 64;
pub const MAX_ANIM_SEQ_LENGTH: usize = 16;

pub struct EntiTilesPlugin;

impl Plugin for EntiTilesPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Update, texture::set_texture_usage);

        app.add_state::<EntiTilesStates>();

        app.add_plugins((EntiTilesTilemapPlugin, EntiTilesRendererPlugin));

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
