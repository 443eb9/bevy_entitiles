use bevy::{prelude::UVec2, reflect::Reflect};

use crate::tilemap::map::Tilemap;

pub mod aabb;
pub mod extension;

#[derive(Debug, Clone, Copy, Reflect)]
pub struct TileArea {
    pub(crate) origin: UVec2,
    pub(crate) extent: UVec2,
    pub(crate) dest: UVec2,
}

impl TileArea {
    /// Define a new fill area.
    ///
    /// Leave `extent` as `None` to fill from the origin to the edge.
    pub fn new(origin: UVec2, extent: Option<UVec2>, tilemap: &Tilemap) -> Self {
        let extent = match extent {
            Some(extent) => {
                if tilemap.is_out_of_tilemap_uvec(origin + extent - 1) {
                    panic!("Part of the fill area is out of the tilemap");
                };
                extent
            }
            None => UVec2 {
                x: tilemap.size.x - origin.x,
                y: tilemap.size.y - origin.y,
            },
        };
        Self {
            origin,
            extent,
            dest: origin + extent - 1,
        }
    }

    /// Define a new fill area without checking if the area is out of the tilemap.
    #[inline]
    pub fn new_unchecked(origin: UVec2, extent: UVec2) -> Self {
        Self {
            origin,
            extent,
            dest: origin + extent - 1,
        }
    }

    /// Define a new fill area that fills the entire tilemap.
    pub fn full(tilemap: &Tilemap) -> Self {
        Self::new(UVec2::ZERO, None, tilemap)
    }

    #[inline]
    pub fn size(&self) -> usize {
        (self.extent.x * self.extent.y) as usize
    }

    #[inline]
    pub fn origin(&self) -> UVec2 {
        self.origin
    }

    #[inline]
    pub fn extent(&self) -> UVec2 {
        self.extent
    }

    #[inline]
    pub fn dest(&self) -> UVec2 {
        self.dest
    }

    pub fn set_extent(&mut self, extent: UVec2, tilemap: &Tilemap) {
        self.extent = extent;
        self.dest = self.origin + extent - 1;

        assert!(
            !tilemap.is_out_of_tilemap_uvec(self.dest),
            "Failed to set extent! The new extent is out of the tilemap!"
        );
    }

    pub fn set_dest(&mut self, dest: UVec2, tilemap: &Tilemap) {
        self.dest = dest;
        self.extent = dest - self.origin + 1;

        assert!(
            !tilemap.is_out_of_tilemap_uvec(self.dest),
            "Failed to set dest! The new dest is out of the tilemap!"
        );
    }

    #[must_use]
    pub fn with_extent(mut self, extent: UVec2, tilemap: &Tilemap) -> Self {
        self.set_extent(extent, tilemap);
        self
    }

    #[must_use]
    pub fn with_dest(mut self, dest: UVec2, tilemap: &Tilemap) -> Self {
        self.set_dest(dest, tilemap);
        self
    }
}
