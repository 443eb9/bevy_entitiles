use std::marker::PhantomData;

use bevy::{
    asset::AssetServer,
    ecs::{
        bundle::Bundle,
        system::{Commands, EntityCommands},
    },
    utils::HashMap,
};

use super::{
    components::{EntityIid, LdtkEntityTempTransform},
    json::{field::FieldInstance, level::EntityInstance},
    resources::{LdtkAssets, LdtkLevelManager},
};

#[derive(Debug, Clone)]
pub struct PackedLdtkEntity {
    pub instance: EntityInstance,
    pub fields: HashMap<String, FieldInstance>,
    pub iid: EntityIid,
    pub transform: LdtkEntityTempTransform,
}

impl PackedLdtkEntity {
    pub fn instantiate(
        self,
        commands: &mut Commands,
        entity_registry: &LdtkEntityRegistry,
        manager: &LdtkLevelManager,
        ldtk_assets: &LdtkAssets,
        asset_server: &AssetServer,
    ) {
        let phantom_entity = {
            if let Some(e) = entity_registry.get(&self.instance.identifier) {
                e
            } else if !manager.ignore_unregistered_entities {
                panic!(
                    "Could not find entity type with entity identifier: {}! \
                    You need to register it using App::register_ldtk_entity::<T>() first!",
                    self.instance.identifier
                );
            } else {
                return;
            }
        };
        let mut entity = commands.spawn_empty();
        phantom_entity.spawn(
            &mut entity,
            &self.instance,
            &self.fields,
            asset_server,
            &manager,
            ldtk_assets,
        )
    }
}

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
