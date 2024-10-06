use std::fmt::Debug;

use bevy::{math::IVec2, reflect::Reflect, utils::HashMap};

use crate::{
    math::GridRect,
    tilemap::tile::{Tile, TileBuilder},
};

/// A marker trait
pub trait Tiles: Debug + Clone + Reflect {}

pub type ColorTileBuffer = TileBuffer<Tile>;
pub type TileBuilderBuffer = TileBuffer<TileBuilder>;
#[cfg(feature = "algorithm")]
pub type PathTileBuffer = TileBuffer<crate::tilemap::algorithm::path::PathTile>;
#[cfg(feature = "physics")]
pub type PhysicsTileBuffer = TileBuffer<crate::tilemap::physics::PhysicsTile>;
#[cfg(feature = "physics")]
pub type PackedPhysicsTileBuffer = TileBuffer<crate::tilemap::physics::PackedPhysicsTile>;

/// A buffer of tiles.
#[derive(Debug, Clone, Reflect)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub struct TileBuffer<T: Tiles> {
    pub(crate) tiles: HashMap<IVec2, T>,
    pub(crate) aabb: GridRect,
}

impl<T: Tiles> TileBuffer<T> {
    pub fn new() -> Self {
        Self {
            tiles: HashMap::new(),
            aabb: GridRect::default(),
        }
    }

    /// Set the tile at the given index. Overwrites the previous tile.
    #[inline]
    pub fn set(&mut self, index: IVec2, tile: T) {
        self.tiles.insert(index, tile);
        self.aabb.union_point(index);
    }

    /// Warning: this method will cause aabb to be recalculated.
    pub fn remove(&mut self, index: IVec2) {
        self.tiles.remove(&index);
        self.recalculate_rect();
    }

    #[inline]
    pub fn get(&self, index: IVec2) -> Option<&T> {
        self.tiles.get(&index)
    }

    #[inline]
    pub fn get_mut(&mut self, index: IVec2) -> Option<&mut T> {
        self.tiles.get_mut(&index)
    }

    /// Recalculate the aabb of this tile buffer.
    ///
    /// This method can be expensive when the tile buffer is large.
    pub fn recalculate_rect(&mut self) {
        self.aabb = GridRect::default();
        for (index, _) in self.tiles.iter() {
            self.aabb = self.aabb.union_point(*index);
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.tiles.is_empty()
    }

    #[inline]
    pub fn aabb(&self) -> GridRect {
        self.aabb
    }
}
