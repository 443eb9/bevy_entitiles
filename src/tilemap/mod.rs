pub mod chunk;

use bevy::prelude::{
    Bundle, Component, ComputedVisibility, GlobalTransform, Handle, IVec2, Image, Transform, UVec2,
    Visibility,
};

#[derive(Default)]
pub enum TileType {
    #[default]
    Square,
}

#[derive(Default)]
pub enum TileTexture {
    #[default]
    None,
    SingleAtlas(Handle<Image>),
}

#[derive(Component, Default)]
pub struct TileMap {
    pub ty: TileType,
    pub size: UVec2,
    pub tile_size: UVec2,
    pub chunk_size: UVec2,
    pub texture: Handle<Image>,
}

#[derive(Bundle, Default)]
pub struct TileMapBundle {
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub tilemap: TileMap,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
}

pub struct Tile {
    pub texture_index: u32,
}

impl TileMap {
    /// Set the tile at the given coordinate.
    pub fn set_tile(&mut self, coordinate: UVec2, tile: Tile) {
        if !self.is_oob(coordinate) {
            self.tiles[coordinate.y * self.size.x + coordinate.x] = tile;
        }
    }

    /// Set the tile at the given coordinate without checking if it is out of bounds.
    pub fn set_tile_unchecked(&mut self, coordinate: UVec2, tile: Tile) {
        self.tiles[coordinate.y * self.size.x + coordinate.x] = tile;
    }

    /// Get the tile at the given coordinate.
    pub fn get_tile(&mut self, coordinate: UVec2) -> Option<Tile> {
        if !self.is_oob(coordinate) {
            Some(self.tiles[coordinate.y * self.size.x + coordinate.x])
        } else {
            None
        }
    }

    /// Get the tile at the given coordinate without checking if it is out of bounds.
    pub fn get_tile_unchecked(&mut self, coordinate: UVec2) -> Tile {
        self.tiles[coordinate.y * self.size.x + coordinate.x]
    }

    /// Check if the given coordinate is out of bounds.
    fn is_oob(&mut self, coordinate: UVec2) -> bool {
        coordinate.x >= self.size.x || coordinate.y >= self.size.y
    }
}
