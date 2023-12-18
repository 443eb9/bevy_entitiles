use std::marker::PhantomData;

use bevy::{
    asset::AssetServer,
    ecs::{bundle::Bundle, entity::Entity, system::EntityCommands},
    sprite::SpriteSheetBundle,
    utils::HashMap,
};

use super::json::field::FieldInstance;

pub type LdtkEntityRegistry = HashMap<String, Box<dyn PhantomLdtkEntityTrait>>;

pub trait LdtkEntity {
    fn initialize(
        level_entity: Entity,
        commands: &mut EntityCommands,
        sprite: Option<SpriteSheetBundle>,
        fields: &HashMap<String, FieldInstance>,
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
        level_entity: Entity,
        commands: &mut EntityCommands,
        sprite: Option<SpriteSheetBundle>,
        fields: &HashMap<String, FieldInstance>,
        asset_server: &AssetServer,
    );
}

impl<T: LdtkEntity + Bundle> PhantomLdtkEntityTrait for PhantomLdtkEntity<T> {
    fn spawn(
        &self,
        level_entity: Entity,
        commands: &mut EntityCommands,
        sprite: Option<SpriteSheetBundle>,
        fields: &HashMap<String, FieldInstance>,
        asset_server: &AssetServer,
    ) {
        T::initialize(level_entity, commands, sprite, fields, asset_server);
    }
}
