use std::marker::PhantomData;

use bevy::{
    asset::AssetServer,
    ecs::{bundle::Bundle, system::EntityCommands},
    sprite::SpriteSheetBundle,
    utils::HashMap,
};

use super::json::level::EntityInstance;

pub type LdtkEntityRegistry = HashMap<String, Box<dyn PhantomLdtkEntityTrait>>;

pub trait LdtkEntity {
    fn initialize(
        commands: &mut EntityCommands,
        sprite: Option<SpriteSheetBundle>,
        entity_instance: &EntityInstance,
        asset_server: &AssetServer,
    );
}

pub struct PhantomLdtkEntity<T: LdtkEntity + Bundle> {
    pub marker: PhantomData<T>,
}

impl<T: LdtkEntity + Bundle> PhantomLdtkEntity<T> {
    pub fn new() -> Self {
        Self {
            marker: PhantomData::<T>,
        }
    }
}

pub trait PhantomLdtkEntityTrait {
    fn spawn(
        &self,
        commands: &mut EntityCommands,
        sprite: Option<SpriteSheetBundle>,
        entity_instance: &EntityInstance,
        asset_server: &AssetServer,
    );
}

impl<T: LdtkEntity + Bundle> PhantomLdtkEntityTrait for PhantomLdtkEntity<T> {
    fn spawn(
        &self,
        commands: &mut EntityCommands,
        sprite: Option<SpriteSheetBundle>,
        entity_instance: &EntityInstance,
        asset_server: &AssetServer,
    ) {
        T::initialize(commands, sprite, entity_instance, asset_server);
    }
}
