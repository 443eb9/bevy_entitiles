use bevy::{
    ecs::{component::Component, entity::Entity, system::Commands},
    math::Vec2,
    prelude::Deref,
    reflect::Reflect,
    utils::HashMap,
};

use crate::ldtk::resources::LdtkGlobalEntityRegistry;

#[derive(Component)]
pub struct LdtkUnloadLayer;

#[derive(Component, Reflect)]
pub struct LdtkLoadedLevel {
    pub identifier: String,
    pub layers: HashMap<LayerIid, Entity>,
    pub entities: HashMap<EntityIid, Entity>,
    pub background: Entity,
}

impl LdtkLoadedLevel {
    pub fn unload(&self, commands: &mut Commands, global_entities: &LdtkGlobalEntityRegistry) {
        self.layers.values().for_each(|e| {
            commands.entity(*e).insert(LdtkUnloadLayer);
        });
        self.entities
            .iter()
            .filter(|(iid, _)| !global_entities.contains_key(*iid))
            .for_each(|(_, e)| {
                commands.entity(*e).despawn();
            });
        commands.entity(self.background).despawn();
    }
}

#[derive(Component, Debug, Clone)]
pub struct LdtkTempTransform {
    pub level_translation: Vec2,
    pub z_index: f32,
}

#[derive(Component, Reflect)]
pub struct GlobalEntity;

#[derive(Component, Debug, Reflect, Hash, Eq, PartialEq, Clone, Deref)]
pub struct EntityIid(pub String);

#[derive(Component, Debug, Reflect, Hash, Eq, PartialEq, Clone, Deref)]
pub struct LayerIid(pub String);

#[derive(Component, Debug, Reflect, Hash, Eq, PartialEq, Clone, Deref)]
pub struct LevelIid(pub String);

#[derive(Component, Debug, Reflect, Hash, Eq, PartialEq, Clone, Deref)]
pub struct WorldIid(pub String);
