use bevy::prelude::UVec2;

use crate::tilemap::map::Tilemap;

pub mod aabb;
pub mod extension;
pub mod tilemap;

#[derive(Debug, Clone, Copy)]
pub struct FillArea {
    pub(crate) origin: UVec2,
    pub(crate) extent: UVec2,
    pub(crate) dest: UVec2,
}

impl FillArea {
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

    /// Define a new fill area that fills the entire tilemap.
    pub fn full(tilemap: &Tilemap) -> Self {
        Self::new(UVec2::ZERO, None, tilemap)
    }

    pub fn size(&self) -> usize {
        (self.extent.x * self.extent.y) as usize
    }
}
