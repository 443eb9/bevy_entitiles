use bevy::prelude::{Plugin, Update};
#[cfg(feature = "debug")]
use debug::EntiTilesDebugPlugin;
use render::{texture::set_texture_usage, EntiTilesRendererPlugin};

pub mod debug;
pub mod math;
pub mod render;
pub mod tilemap;

pub struct EntiTilesPlugin;

impl Plugin for EntiTilesPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Update, set_texture_usage);

        app.add_plugins(EntiTilesRendererPlugin);

        #[cfg(feature = "debug")]
        app.add_plugins(EntiTilesDebugPlugin);
    }
}
