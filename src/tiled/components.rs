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
    pub name: String,
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

/// A component that is used to mark a tilemap as a global object.
/// 
/// Global objects means objects that are not attached to any tilemap.
/// So they won't be unloaded when the tilemap is unloaded.
#[derive(Component, Debug, Clone)]
pub struct TiledGlobalObject;
