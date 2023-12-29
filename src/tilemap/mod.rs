use bevy::app::Plugin;

#[cfg(feature = "algorithm")]
pub mod algorithm;
pub mod layer;
pub mod map;
#[cfg(any(feature = "physics_xpbd", feature = "physics_rapier"))]
pub mod physics;
pub mod tile;

pub struct EntiTilesTilemapPlugin;

impl Plugin for EntiTilesTilemapPlugin {
    #[allow(unused_variables)]
    fn build(&self, app: &mut bevy::prelude::App) {
        #[cfg(feature = "algorithm")]
        app.add_plugins(algorithm::EntiTilesAlgorithmTilemapPlugin);
        #[cfg(any(feature = "physics_rapier", feature = "physics_xpbd"))]
        app.add_plugins(physics::EntiTilesPhysicsTilemapPlugin);
    }
}
