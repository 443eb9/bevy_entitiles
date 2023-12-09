use bevy::{
    ecs::{component::Component, entity::Entity},
    math::UVec2,
    utils::HashMap,
};

use crate::{algorithm::pathfinding::PathTile, math::FillArea};

use crate::tilemap::map::Tilemap;

#[derive(Component)]
pub struct PathTilemap {
    #[allow(unused)]
    // TODO: do something with this.
    // I mean I have no idea what to do with this so far.
    pub(crate) tilemap: Entity,
    pub(crate) tiles: HashMap<UVec2, PathTile>,
}

impl PathTilemap {
    pub fn new(tilemap: Entity) -> Self {
        Self {
            tilemap,
            tiles: HashMap::default(),
        }
    }

    pub fn get_tile(&self, index: UVec2) -> Option<&PathTile> {
        self.tiles.get(&index)
    }

    pub fn get_tile_mut(&mut self, index: UVec2) -> Option<&mut PathTile> {
        self.tiles.get_mut(&index)
    }

    pub fn update_tile(&mut self, index: UVec2, new_tile: PathTile) {
        self.tiles.insert(index, new_tile);
    }

    /// Set path-finding data using a custom function.
    /// Before fill path, you need to fill the tiles first.
    /// Those empty indices will be ignored.
    pub fn fill_path_rect_custom(
        &mut self,
        tilemap: &Tilemap,
        area: FillArea,
        mut path_tile: impl FnMut(UVec2) -> PathTile,
    ) {
        for y in area.origin.y..=area.dest.y {
            for x in area.origin.x..=area.dest.x {
                let idx = UVec2::new(x, y);
                if tilemap.get_unchecked(idx).is_some() {
                    self.tiles.insert(idx, path_tile(idx));
                }
            }
        }
    }

    /// Fill path-finding data using `PathTile`.
    /// Before fill path, you need to fill the tiles first.
    /// Those empty indices will be ignored.
    pub fn fill_path_rect(&mut self, tilemap: &Tilemap, area: FillArea, path_tile: &PathTile) {
        for y in area.origin.y..=area.dest.y {
            for x in area.origin.x..=area.dest.x {
                let idx = UVec2::new(x, y);
                if tilemap.get_unchecked(idx).is_some() {
                    self.tiles.insert(idx, path_tile.clone());
                }
            }
        }
    }
}
