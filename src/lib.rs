use bevy::prelude::{Plugin, Update};
use render::{texture::set_texture_usage, EntiTilesRendererPlugin};

pub mod math;
pub mod render;
pub mod tilemap;

pub struct EntiTilesPlugin;

impl Plugin for EntiTilesPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Update, set_texture_usage);

        app.add_plugins(EntiTilesRendererPlugin);
    }
}
