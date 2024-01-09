use bevy::app::{Plugin, Update};

#[cfg(feature = "algorithm")]
pub mod algorithm;
pub mod buffers;
pub mod bundles;
pub mod coordinates;
pub mod map;
#[cfg(any(feature = "physics_xpbd", feature = "physics_rapier"))]
pub mod physics;
pub mod storage;
pub mod tile;

pub struct EntiTilesTilemapPlugin;

impl Plugin for EntiTilesTilemapPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Update, (map::transform_syncer, tile::tile_updater));

        #[cfg(feature = "algorithm")]
        app.add_plugins(algorithm::EntiTilesAlgorithmTilemapPlugin);
        #[cfg(any(feature = "physics_rapier", feature = "physics_xpbd"))]
        app.add_plugins(physics::EntiTilesPhysicsTilemapPlugin);
    }
}
