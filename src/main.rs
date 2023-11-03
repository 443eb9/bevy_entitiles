use bevy::prelude::*;
use bevy_entitiles::{render::EntiTilesRendererPlugin, EntiTilesPlugin};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EntiTilesPlugin, EntiTilesRendererPlugin))
        .insert_resource(Msaa::Off)
        .run();
}
