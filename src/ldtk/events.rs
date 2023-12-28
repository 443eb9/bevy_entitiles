use bevy::{ecs::event::Event, reflect::Reflect};

#[derive(Event)]
pub enum LdtkEvent {
    LevelLoaded(LevelEvent),
    LevelUnloaded(LevelEvent),
}

#[derive(Reflect, Debug, Clone)]
pub struct LevelEvent {
    pub identifier: String,
    pub iid: String,
}
