use bevy::app::Plugin;

use crate::tilemap::algorithm::path::{PathTile, PathTilemap};

pub mod path;

pub struct EntiTilesAlgorithmTilemapPlugin;

impl Plugin for EntiTilesAlgorithmTilemapPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_type::<PathTilemap>()
            .register_type::<PathTile>();
    }
}
