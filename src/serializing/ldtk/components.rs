use bevy::ecs::component::Component;

#[derive(Component)]
pub struct LdtkLoadedLevel {
    pub identifier: String,
    pub iid: String,
}
