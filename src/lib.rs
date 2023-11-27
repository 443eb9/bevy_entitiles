use bevy::{app::Update, prelude::Plugin};
use render::{texture, EntiTilesRendererPlugin};

#[cfg(feature = "algorithm")]
pub mod algorithm;
#[cfg(feature = "debug")]
pub mod debug;
pub mod math;
#[cfg(feature = "post_processing")]
pub mod post_processing;
pub mod render;
pub mod tilemap;

pub struct EntiTilesPlugin;

impl Plugin for EntiTilesPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Update, texture::set_texture_usage);

        app.add_plugins(EntiTilesRendererPlugin);

        #[cfg(feature = "algorithm")]
        app.add_plugins(algorithm::EntitilesAlgorithmPlugin);
        #[cfg(any(feature = "physics_rapier", feature = "physics_xpbd"))]
        app.add_plugins(tilemap::physics::TilemapPhysicsPlugin);
        #[cfg(feature = "post_processing")]
        app.add_plugins(post_processing::EntiTilesPostProcessingPlugin);
    }
}
