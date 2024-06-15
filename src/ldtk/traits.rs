use std::marker::PhantomData;

use bevy::{
    asset::AssetServer,
    ecs::{bundle::Bundle, component::Component, system::EntityCommands},
    utils::HashMap,
};

use crate::ldtk::{
    json::{field::FieldInstance, level::EntityInstance},
    resources::LdtkAssets,
};

pub type LdtkEntityRegistry = HashMap<String, Box<dyn PhantomLdtkEntityTrait>>;

pub trait LdtkEntity {
    fn initialize(
        commands: &mut EntityCommands,
        entity_instance: &EntityInstance,
        fields: &HashMap<String, FieldInstance>,
        asset_server: &AssetServer,
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
        ldtk_assets: &LdtkAssets,
    ) {
        T::initialize(commands, entity_instance, fields, asset_server, ldtk_assets);
    }
}

pub trait LdtkEnum {
    fn get_identifier(ident: &str) -> Self;
}

pub type LdtkEntityTagRegistry = HashMap<String, Box<dyn PhantomLdtkEntityTagTrait>>;

pub trait LdtkEntityTag {
    fn add_tag(commands: &mut EntityCommands);
}

pub struct PhantomLdtkEntityTag<T: LdtkEntityTag + Component> {
    pub marker: PhantomData<T>,
}

impl<T: LdtkEntityTag + Component> PhantomLdtkEntityTag<T> {
    pub fn new() -> Self {
        Self {
            marker: PhantomData::<T>,
        }
    }
}

pub trait PhantomLdtkEntityTagTrait {
    fn add_tag(&self, commands: &mut EntityCommands);
}

impl<T: LdtkEntityTag + Component> PhantomLdtkEntityTagTrait for PhantomLdtkEntityTag<T> {
    fn add_tag(&self, commands: &mut EntityCommands) {
        T::add_tag(commands);
    }
}
