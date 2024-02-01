use std::fmt::Debug;

use bevy::{math::IVec2, reflect::Reflect, utils::HashMap};

use crate::math::aabb::IAabb2d;

use super::tile::{Tile, TileBuilder};

/// A marker trait
pub trait Tiles: Debug + Clone + Reflect {}

pub type ColorTileBuffer = TileBuffer<Tile>;
pub type TileBuilderBuffer = TileBuffer<TileBuilder>;
#[cfg(feature = "algorithm")]
pub type PathTileBuffer = TileBuffer<super::algorithm::path::PathTile>;
#[cfg(feature = "physics")]
pub type PhysicsTileBuffer = TileBuffer<super::physics::PhysicsTile>;
#[cfg(feature = "physics")]
pub type PackedPhysicsTileBuffer = TileBuffer<super::physics::PackedPhysicsTile>;

/// A buffer of tiles.
#[derive(Debug, Clone, Reflect)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub struct TileBuffer<T: Tiles> {
    pub(crate) tiles: HashMap<IVec2, T>,
    pub(crate) aabb: IAabb2d,
}

impl<T: Tiles> TileBuffer<T> {
    pub fn new() -> Self {
        Self {
            tiles: HashMap::new(),
            aabb: IAabb2d::default(),
        }
    }

    /// Set the tile at the given index. Overwrites the previous tile.
    #[inline]
    pub fn set(&mut self, index: IVec2, tile: T) {
        self.tiles.insert(index, tile);
        self.aabb.expand_to_contain(index);
    }

    /// Warning: this method will cause aabb to be recalculated.
    pub fn remove(&mut self, index: IVec2) {
        self.tiles.remove(&index);
        self.recalculate_aabb();
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
    pub fn recalculate_aabb(&mut self) {
        self.aabb = IAabb2d::default();
        for (index, _) in self.tiles.iter() {
            self.aabb.expand_to_contain(*index);
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.tiles.is_empty()
    }

    #[inline]
    pub fn aabb(&self) -> IAabb2d {
        self.aabb
    }
}
