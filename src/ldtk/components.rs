use bevy::{ecs::component::Component, reflect::Reflect};

#[derive(Component, Reflect)]
pub struct LdtkLoadedLevel {
    pub identifier: String,
    pub iid: String,
}

#[derive(Component, Reflect)]
pub struct GlobalEntity;

#[derive(Component, Reflect, Hash, Eq, PartialEq, Clone)]
pub struct EntityIid(pub String);

#[derive(Component, Reflect)]
pub struct LayerIid(pub String);

#[derive(Component, Reflect)]
pub struct LevelIid(pub String);

#[derive(Component, Reflect)]
pub struct WorldIid(pub String);
