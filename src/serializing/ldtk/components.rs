use bevy::ecs::component::Component;

#[derive(Component)]
pub struct EntityIid(pub String);

#[derive(Component)]
pub struct LayerIid(pub String);

#[derive(Component)]
pub struct LevelIid(pub String);

#[derive(Component)]
pub struct WorldIid(pub String);
