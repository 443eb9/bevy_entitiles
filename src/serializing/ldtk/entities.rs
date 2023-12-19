use std::marker::PhantomData;

use bevy::{
    asset::AssetServer,
    ecs::{bundle::Bundle, entity::Entity, system::EntityCommands},
    utils::HashMap,
};

use super::{
    json::{field::FieldInstance, level::EntityInstance},
    resources::LdtkTextures,
};

pub type LdtkEntityRegistry = HashMap<String, Box<dyn PhantomLdtkEntityTrait>>;

pub trait LdtkEntity {
    fn initialize(
        level_entity: Entity,
        commands: &mut EntityCommands,
        entity_instance: &EntityInstance,
        fields: &HashMap<String, FieldInstance>,
        asset_server: &AssetServer,
        ldtk_textures: &LdtkTextures,
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
        entity_instance: &EntityInstance,
        fields: &HashMap<String, FieldInstance>,
        asset_server: &AssetServer,
        ldtk_textures: &LdtkTextures,
    );
}

impl<T: LdtkEntity + Bundle> PhantomLdtkEntityTrait for PhantomLdtkEntity<T> {
    fn spawn(
        &self,
        level_entity: Entity,
        commands: &mut EntityCommands,
        entity_instance: &EntityInstance,
        fields: &HashMap<String, FieldInstance>,
        asset_server: &AssetServer,
        ldtk_textures: &LdtkTextures,
    ) {
        T::initialize(
            level_entity,
            commands,
            entity_instance,
            fields,
            asset_server,
            ldtk_textures,
        );
    }
}
