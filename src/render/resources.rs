use bevy::{
    asset::AssetId,
    ecs::{entity::EntityHashMap, system::Resource},
};

use super::{extract::ExtractedTilemap, material::TilemapMaterial};

#[derive(Resource)]
pub struct TilemapInstances<M: TilemapMaterial>(pub EntityHashMap<ExtractedTilemap<M>>);

impl<M: TilemapMaterial> Default for TilemapInstances<M> {
    fn default() -> Self {
        Self(Default::default())
    }
}

#[derive(Resource)]
pub struct ExtractedTilemapMaterials<M: TilemapMaterial> {
    pub changed: Vec<(AssetId<M>, M)>,
    pub removed: Vec<AssetId<M>>,
}

impl<M: TilemapMaterial> Default for ExtractedTilemapMaterials<M> {
    fn default() -> Self {
        Self {
            changed: Default::default(),
            removed: Default::default(),
        }
    }
}
