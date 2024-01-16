use std::marker::PhantomData;

use bevy::{
    asset::AssetServer,
    ecs::{bundle::Bundle, component::Component, system::EntityCommands},
    utils::HashMap,
};

use super::{
    json::{field::FieldInstance, level::EntityInstance},
    resources::{LdtkAssets, LdtkLevelManager},
};

pub type LdtkEntityRegistry = HashMap<String, Box<dyn PhantomLdtkEntityTrait>>;

pub trait LdtkEntity {
    fn initialize(
        commands: &mut EntityCommands,
        entity_instance: &EntityInstance,
        fields: &HashMap<String, FieldInstance>,
        asset_server: &AssetServer,
        ldtk_manager: &LdtkLevelManager,
        ldtk_assets: &LdtkAssets,
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
        entity_instance: &EntityInstance,
        fields: &HashMap<String, FieldInstance>,
        asset_server: &AssetServer,
        ldtk_manager: &LdtkLevelManager,
        assets: &LdtkAssets,
    );
}

impl<T: LdtkEntity + Bundle> PhantomLdtkEntityTrait for PhantomLdtkEntity<T> {
    fn spawn(
        &self,
        commands: &mut EntityCommands,
        entity_instance: &EntityInstance,
        fields: &HashMap<String, FieldInstance>,
        asset_server: &AssetServer,
        ldtk_manager: &LdtkLevelManager,
        ldtk_assets: &LdtkAssets,
    ) {
        T::initialize(
            commands,
            entity_instance,
            fields,
            asset_server,
            ldtk_manager,
            ldtk_assets,
        );
    }
}

pub trait LdtkEnum {
    fn get_identifier(ident: &str) -> Self;
}

pub struct LdtkEntityTag(pub Box<dyn PhantomLdtkEntityTagTrait>);

pub struct PhantomLdtkEntityTag<T: LdtkEnum + Component> {
    pub marker: PhantomData<T>,
}

impl<T: LdtkEnum + Component> PhantomLdtkEntityTag<T> {
    pub fn new() -> Self {
        Self {
            marker: PhantomData::<T>,
        }
    }
}

pub trait PhantomLdtkEntityTagTrait {
    fn add_tag(&self, commands: &mut EntityCommands, ident: String);
}

impl<T: LdtkEnum + Component> PhantomLdtkEntityTagTrait for PhantomLdtkEntityTag<T> {
    fn add_tag(&self, commands: &mut EntityCommands, ident: String) {
        commands.insert(T::get_identifier(&ident));
    }
}
