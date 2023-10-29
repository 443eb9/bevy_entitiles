use bevy::prelude::{Plugin, Startup};
use test::*;

pub mod math;
pub mod render;
pub mod test;
pub mod tilemap;

pub struct EntiTilesPlugin;

impl Plugin for EntiTilesPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, random_tests);
    }
}
