use bevy::{
    asset::AssetId,
    ecs::{entity::Entity, system::Resource},
    utils::EntityHashMap,
};

use super::{extract::ExtractedTilemap, material::TilemapMaterial};

#[derive(Resource, Default)]
pub struct TilemapInstances(pub EntityHashMap<Entity, ExtractedTilemap>);

#[derive(Resource, Default)]
pub struct TilemapMaterialInstances<M: TilemapMaterial>(pub EntityHashMap<Entity, AssetId<M>>);
