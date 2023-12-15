use std::marker::PhantomData;

use bevy::{
    asset::AssetServer,
    ecs::{bundle::Bundle, system::EntityCommands},
    utils::HashMap,
};

use super::json::level::EntityInstance;

pub type LdtkEntityIdentMapper = HashMap<String, Box<dyn LdtkEntityTypeMarkerTrait>>;

pub trait LdtkEntity {
    fn initialize(
        commands: &mut EntityCommands,
        entity_instance: &EntityInstance,
        asset_server: &AssetServer,
    ) -> Self;
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
    fn spawn(
        &self,
        commands: &mut EntityCommands,
        entity_instance: &EntityInstance,
        asset_server: &AssetServer,
    );
}

impl<T: LdtkEntity + Bundle> LdtkEntityTypeMarkerTrait for LdtkEntityTypeMarker<T> {
    fn spawn(
        &self,
        commands: &mut EntityCommands,
        entity_instance: &EntityInstance,
        asset_server: &AssetServer,
    ) {
        let b = T::initialize(commands, entity_instance, asset_server);
        commands.insert(b);
    }
}
