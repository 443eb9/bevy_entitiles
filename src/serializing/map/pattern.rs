use bevy::{ecs::system::Commands, math::IVec2, reflect::Reflect, utils::HashMap};
use serde::{Deserialize, Serialize};

use crate::{
    math::aabb::IAabb2d,
    tilemap::{map::TilemapStorage, tile::TileBuffer},
};

use super::SerializedTile;

#[derive(Serialize, Deserialize, Debug, Clone, Reflect)]
pub struct TilemapPattern {
    pub(crate) label: Option<String>,
    pub(crate) tiles: HashMap<IVec2, SerializedTile>,
    #[cfg(feature = "algorithm")]
    pub(crate) path_tiles: Option<HashMap<IVec2, crate::tilemap::algorithm::path::PathTile>>,
    pub(crate) aabb: IAabb2d,
}

impl TilemapPattern {
    pub fn new(label: Option<String>) -> Self {
        Self {
            label,
            tiles: HashMap::new(),
            #[cfg(feature = "algorithm")]
            path_tiles: None,
            aabb: IAabb2d::default(),
        }
    }

    pub fn get(&self, index: IVec2) -> Option<&SerializedTile> {
        self.tiles.get(&index)
    }

    pub fn get_mut(&mut self, index: IVec2) -> Option<&mut SerializedTile> {
        self.tiles.get_mut(&index)
    }

    pub fn set(&mut self, index: IVec2, tile: SerializedTile) {
        self.tiles.insert(index, tile);
        self.aabb.expand_to_contain(index);
    }

    pub fn remove(&mut self, index: IVec2) {
        self.tiles.remove(&index);
        self.recalculate_aabb();
    }

    pub fn recalculate_aabb(&mut self) {
        self.aabb = IAabb2d::default();
        for (index, _) in self.tiles.iter() {
            self.aabb.expand_to_contain(*index);
        }
    }

    pub fn is_index_oob(&self, index: IVec2) -> bool {
        !self.aabb.contains(index)
    }

    pub fn apply_tiles(&self, commands: &mut Commands, origin: IVec2, target: &mut TilemapStorage) {
        target.fill_with_buffer(commands, origin, self.clone().into());
    }

    #[cfg(feature = "algorithm")]
    pub fn apply_path_tiles(
        &self,
        origin: IVec2,
        target: &mut crate::tilemap::algorithm::path::PathTilemap,
    ) {
        if let Some(path_tiles) = &self.path_tiles {
            path_tiles.iter().for_each(|(index, tile)| {
                target.set(origin + *index, Some(*tile));
            });
        }
    }
}

impl Into<TileBuffer> for TilemapPattern {
    fn into(self) -> TileBuffer {
        TileBuffer {
            tiles: self.tiles.into_iter().map(|t| (t.0, t.1.into())).collect(),
            aabb: self.aabb,
        }
    }
}
