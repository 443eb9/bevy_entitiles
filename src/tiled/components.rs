use bevy::{
    ecs::{component::Component, entity::Entity, system::Commands},
    math::Vec2,
    utils::HashMap,
};

#[derive(Component, Debug, Clone)]
pub struct TiledLoader {
    pub map: String,
    pub trans_ovrd: Option<Vec2>,
}

#[derive(Component, Debug, Clone)]
pub struct TiledUnloader;

#[derive(Component, Debug, Clone)]
pub struct TiledUnloadLayer;

#[derive(Component, Debug, Clone)]
pub struct TiledLoadedTilemap {
    pub map: String,
    pub layers: HashMap<u32, Entity>,
    pub objects: HashMap<u32, Entity>,
}

impl TiledLoadedTilemap {
    pub fn unload(&self, commands: &mut Commands) {
        self.layers.values().for_each(|e| {
            commands.entity(*e).insert(TiledUnloadLayer);
        });
        self.objects.values().for_each(|e| {
            commands.entity(*e).despawn();
        });
    }
}

#[derive(Component, Debug, Clone)]
pub struct TiledGlobalObject;
