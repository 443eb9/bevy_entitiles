use bevy::{math::IVec2, prelude::UVec2, reflect::Reflect};

pub mod aabb;
pub mod extension;

#[derive(Debug, Clone, Copy, Reflect)]
pub struct TileArea {
    pub origin: IVec2,
    pub extent: UVec2,
    pub dest: IVec2,
}

impl TileArea {
    /// Define a new fill area without checking if the area is out of the tilemap.
    #[inline]
    pub fn new(origin: IVec2, extent: UVec2) -> Self {
        Self {
            origin,
            extent,
            dest: origin + extent.as_ivec2() - 1,
        }
    }

    #[inline]
    pub fn size(&self) -> usize {
        (self.extent.x * self.extent.y) as usize
    }
}
