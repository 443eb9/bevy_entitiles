use bevy::{ecs::component::Component, math::Vec2, reflect::Reflect};

#[derive(Component, Reflect)]
pub struct LdtkLoadedLevel {
    pub identifier: String,
    pub iid: String,
}

#[derive(Component)]
pub struct LdtkEntityTempTransform {
    pub level_translation: Vec2,
    pub z_index: f32,
}

#[derive(Component, Reflect)]
pub struct GlobalEntity;

#[derive(Component, Reflect, Hash, Eq, PartialEq, Clone)]
pub struct EntityIid(pub String);

#[derive(Component, Reflect, Hash, Eq, PartialEq, Clone)]
pub struct LayerIid(pub String);

#[derive(Component, Reflect, Hash, Eq, PartialEq, Clone)]
pub struct LevelIid(pub String);

#[derive(Component, Reflect, Hash, Eq, PartialEq, Clone)]
pub struct WorldIid(pub String);
