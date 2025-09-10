use std::collections::HashMap;

use bevy::{prelude::Resource, reflect::Reflect};

use crate::tilemap::physics::PhysicsTile;

#[derive(Debug, Resource, Clone, Reflect)]
pub struct LdtkPhysicsLayer {
    pub identifier: String,
    pub parent: String,
    pub air: i32,
    pub tiles: Option<HashMap<i32, PhysicsTile>>,
}
