use std::{marker::PhantomData, sync::RwLock};

use bevy::{
    ecs::{
        bundle::Bundle,
        system::{EntityCommands, Resource},
    },
    utils::HashMap,
};

use super::json::level::EntityInstance;

pub type LdtkEntityIdentMapper = HashMap<String, Box<dyn LdtkEntityTypeMarkerTrait>>;

pub trait LdtkEntity {
    fn spawn(commands: &mut EntityCommands, entity_instance: EntityInstance) -> Self;
}

pub struct LdtkEntityTypeMarker<T: LdtkEntity + Bundle> {
    pub marker: PhantomData<T>,
}

impl<T: LdtkEntity + Bundle> LdtkEntityTypeMarker<T> {
    pub fn new() -> Self {
        Self {
            marker: PhantomData::<T>,
        }
    }
}

pub trait LdtkEntityTypeMarkerTrait {
    fn spawn(&self, commands: &mut EntityCommands, entity_instance: EntityInstance);
}

impl<T: LdtkEntity + Bundle> LdtkEntityTypeMarkerTrait for LdtkEntityTypeMarker<T> {
    fn spawn(&self, commands: &mut EntityCommands, entity_instance: EntityInstance) {
        T::spawn(commands, entity_instance);
    }
}
