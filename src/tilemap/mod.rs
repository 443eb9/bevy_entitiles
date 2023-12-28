use bevy::app::{Plugin, Update};

use self::layer::{layer_inserter, layer_updater};

#[cfg(feature = "algorithm")]
pub mod algorithm;
pub mod layer;
pub mod map;
#[cfg(any(feature = "physics_xpbd", feature = "physics_rapier"))]
pub mod physics;
pub mod tile;

pub struct EntiTilesTilemapPlugin;

impl Plugin for EntiTilesTilemapPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Update, (layer_updater, layer_inserter));

        #[cfg(feature = "algorithm")]
        app.add_plugins(algorithm::EntiTilesAlgorithmTilemapPlugin);
        #[cfg(any(feature = "physics_rapier", feature = "physics_xpbd"))]
        app.add_plugins(physics::EntiTilesPhysicsTilemapPlugin);
    }
}
