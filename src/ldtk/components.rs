use bevy::{
    ecs::{component::Component, entity::Entity, system::Commands},
    math::Vec2,
    reflect::Reflect,
    utils::HashMap,
};

#[derive(Reflect, Default, Clone, Copy, PartialEq, Eq)]
pub enum LdtkLoaderMode {
    #[default]
    Tilemap,
    MapPattern,
}

#[derive(Component, Reflect, Default)]
pub struct LdtkLoader {
    pub(crate) level: String,
    pub(crate) mode: LdtkLoaderMode,
    pub(crate) trans_ovrd: Option<Vec2>,
}

#[derive(Component, Reflect, Default)]
pub struct LdtkUnloader;

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
    pub fn unload(&self, commands: &mut Commands) {
        self.layers.values().for_each(|e| {
            commands.entity(*e).insert(LdtkUnloadLayer);
        });
        self.entities.values().for_each(|e| {
            commands.entity(*e).despawn();
        });
        commands.entity(self.background).despawn();
    }
}

#[derive(Component, Debug, Clone)]
pub struct LdtkEntityTempTransform {
    pub level_translation: Vec2,
    pub z_index: f32,
}

#[derive(Component, Reflect)]
pub struct GlobalEntity;

#[derive(Component, Debug, Reflect, Hash, Eq, PartialEq, Clone)]
pub struct EntityIid(pub String);

#[derive(Component, Debug, Reflect, Hash, Eq, PartialEq, Clone)]
pub struct LayerIid(pub String);

#[derive(Component, Debug, Reflect, Hash, Eq, PartialEq, Clone)]
pub struct LevelIid(pub String);

#[derive(Component, Debug, Reflect, Hash, Eq, PartialEq, Clone)]
pub struct WorldIid(pub String);
