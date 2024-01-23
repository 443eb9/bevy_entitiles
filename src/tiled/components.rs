use bevy::{ecs::component::Component, math::Vec2};

#[derive(Component, Debug, Clone)]
pub struct TiledLoader {
    pub map: String,
    pub trans_ovrd: Option<Vec2>,
}

#[derive(Component, Debug, Clone)]
pub struct TiledUnloader;
