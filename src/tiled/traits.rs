use std::marker::PhantomData;

use bevy::{
    asset::AssetServer,
    ecs::{bundle::Bundle, system::EntityCommands},
    utils::HashMap,
};

use super::{
    resources::TiledAssets,
    xml::{layer::TiledObjectInstance, property::ClassInstance},
};

pub type TiledObjectRegistry = HashMap<String, Box<dyn PhantomTiledObjectTrait>>;

pub trait TiledObject {
    fn initialize(
        commands: &mut EntityCommands,
        object_instance: &TiledObjectInstance,
        components: &HashMap<String, ClassInstance>,
        asset_server: &AssetServer,
        tiled_assets: &TiledAssets,
        tiled_map: String,
    );
}

pub struct PhantomTiledObject<T: TiledObject + Bundle> {
    marker: PhantomData<T>,
}

impl<T: TiledObject + Bundle> PhantomTiledObject<T> {
    pub fn new() -> Self {
        Self {
            marker: PhantomData,
        }
    }
}

pub trait PhantomTiledObjectTrait {
    fn initialize(
        &self,
        commands: &mut EntityCommands,
        object_instance: &TiledObjectInstance,
        components: &HashMap<String, ClassInstance>,
        asset_server: &AssetServer,
        tiled_assets: &TiledAssets,
        tiled_map: String,
    );
}

impl<T: TiledObject + Bundle> PhantomTiledObjectTrait for PhantomTiledObject<T> {
    fn initialize(
        &self,
        commands: &mut EntityCommands,
        object_instance: &TiledObjectInstance,
        components: &HashMap<String, ClassInstance>,
        asset_server: &AssetServer,
        tiled_assets: &TiledAssets,
        tiled_map: String,
    ) {
        T::initialize(
            commands,
            object_instance,
            components,
            asset_server,
            tiled_assets,
            tiled_map,
        );
    }
}

pub trait TiledEnum {
    fn get_identifier(ident: &str) -> Self;
}
