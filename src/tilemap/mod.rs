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
    fn build(&self, _app: &mut bevy::prelude::App) {}
}
