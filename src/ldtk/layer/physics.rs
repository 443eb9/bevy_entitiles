use bevy::{reflect::Reflect, utils::HashMap};

use crate::tilemap::physics::PhysicsTile;

#[derive(Debug, Clone, Reflect)]
pub struct LdtkPhysicsLayer {
    pub identifier: String,
    pub parent: String,
    pub air: i32,
    pub tiles: Option<HashMap<i32, PhysicsTile>>,
}
