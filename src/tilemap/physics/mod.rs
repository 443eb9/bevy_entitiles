use bevy::{ecs::{entity::Entity, event::Event}, math::UVec2, app::Plugin};

use super::Tile;

#[cfg(feature = "physics_xpbd")]
pub mod xpbd;
#[cfg(feature = "physics_rapier")]
pub mod rapier;

pub struct TilemapPhysicsPlugin;

impl Plugin for TilemapPhysicsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<TileCollision>();

        #[cfg(feature = "physics_rapier")]
        app.add_plugins(crate::tilemap::physics::rapier::PhysicsRapierTilemapPlugin);
        #[cfg(feature = "physics_xpbd")]
        app.add_plugins(crate::tilemap::physics::xpbd::PhysicsXpbdTilemapPlugin);
    }
}

#[derive(Event, Debug)]
pub struct TileCollision {
    pub tile_index: UVec2,
    pub tile_entity: Entity,
    pub tile_snapshot: Tile,
    pub collider_entity: Entity,
}
