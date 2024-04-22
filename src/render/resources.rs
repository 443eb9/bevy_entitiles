use bevy::ecs::{entity::EntityHashMap, system::Resource};

use super::extract::ExtractedTilemap;

#[derive(Resource)]
pub struct TilemapInstances(pub EntityHashMap<ExtractedTilemap>);

impl Default for TilemapInstances {
    fn default() -> Self {
        Self(Default::default())
    }
}
