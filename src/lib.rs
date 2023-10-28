use bevy::{
    prelude::{Plugin, Update},
};
use test::*;

pub mod math;
pub mod render;
pub mod test;
pub mod tilemap;

pub struct EntiTilesPlugin;

impl Plugin for EntiTilesPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Update, random_tests);
    }
}
