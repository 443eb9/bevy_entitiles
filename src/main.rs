use bevy::prelude::*;
use bevy_entitiles::{EntiTilesPlugin, render::EntiTilesRendererPlugin};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EntiTilesPlugin, EntiTilesRendererPlugin))
        .run();
}
