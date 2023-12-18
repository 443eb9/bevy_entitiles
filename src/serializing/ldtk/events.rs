use bevy::ecs::event::Event;

#[derive(Event)]
pub enum LdtkEvent {
    LevelLoaded(LevelEvent),
    LevelUnloaded(LevelEvent),
}

pub struct LevelEvent {
    pub identifier: String,
    pub iid: String,
}
