use bevy::{
    ecs::{entity::Entity, system::Resource},
    utils::EntityHashMap,
};

use super::extract::ExtractedTilemap;

#[derive(Resource, Default)]
pub struct TilemapInstances(EntityHashMap<Entity, ExtractedTilemap>);

impl TilemapInstances {
    #[inline]
    pub fn insert(&mut self, entity: Entity, tilemap: ExtractedTilemap) {
        self.0.insert(entity, tilemap);
    }

    #[inline]
    pub fn get(&self, entity: Entity) -> Option<&ExtractedTilemap> {
        self.0.get(&entity)
    }

    #[inline]
    pub fn get_unwrap(&self, entity: Entity) -> &ExtractedTilemap {
        self.0.get(&entity).unwrap()
    }

    #[inline]
    pub fn remove(&mut self, entity: Entity) {
        self.0.remove(&entity);
    }
}
