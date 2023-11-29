use bevy::{
    app::{Plugin, Update},
    math::{UVec2, Vec2, Vec4},
    render::render_resource::FilterMode,
    utils::HashMap,
};
use serde::{Deserialize, Serialize};

use crate::{
    math::aabb::AabbBox2d,
    render::texture::{TileUV, TilemapTextureDescriptor},
    tilemap::{
        map::Tilemap,
        tile::{Tile, TileType},
    },
};

use self::save::{save, TilemapSaver};

pub mod save;

pub struct EntitilesSerializingPlugin;

impl Plugin for EntitilesSerializingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Update, save);
    }
}

#[derive(Serialize, Deserialize)]
pub struct SerializedTilemapData {
    pub tilemap: SerializedTilemap,
    pub tiles: Vec<SerializedTile>,
}

#[derive(Serialize, Deserialize)]
pub struct SerializedTilemap {
    pub tile_type: TileType,
    pub size: UVec2,
    pub tile_render_scale: Vec2,
    pub tile_slot_size: Vec2,
    pub anchor: Vec2,
    pub render_chunk_size: u32,
    pub texture: Option<SerializedTilemapTexture>,
    pub flip: u32,
    pub aabb: AabbBox2d,
    pub translation: Vec2,
    pub z_order: u32,
}

impl SerializedTilemap {
    pub fn from_tilemap(tilemap: &Tilemap, saver: &TilemapSaver) -> Self {
        SerializedTilemap {
            tile_type: tilemap.tile_type,
            size: tilemap.size,
            tile_render_scale: tilemap.tile_render_scale,
            tile_slot_size: tilemap.tile_slot_size,
            anchor: tilemap.anchor,
            render_chunk_size: tilemap.render_chunk_size,
            texture: if let Some(tex) = &saver.texture_path {
                Some(SerializedTilemapTexture {
                    path: tex.clone(),
                    desc: tilemap.texture.clone().unwrap().desc.into(),
                })
            } else {
                None
            },
            flip: tilemap.flip,
            aabb: tilemap.aabb,
            translation: tilemap.translation,
            z_order: tilemap.z_order,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SerializedTilemapTexture {
    pub path: String,
    pub desc: SerializedTilemapDescriptor,
}

#[derive(Serialize, Deserialize)]
pub struct SerializedTilemapDescriptor {
    pub size: UVec2,
    pub tiles_uv: Vec<TileUV>,
    pub filter_mode: SerializedFilterMode,
    pub is_uniform: bool,
}

impl From<TilemapTextureDescriptor> for SerializedTilemapDescriptor {
    fn from(value: TilemapTextureDescriptor) -> Self {
        Self {
            size: value.size,
            tiles_uv: value.tiles_uv,
            filter_mode: value.filter_mode.into(),
            is_uniform: value.is_uniform,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum SerializedFilterMode {
    Nearest = 0,
    Linear = 1,
}

impl From<FilterMode> for SerializedFilterMode {
    fn from(value: FilterMode) -> Self {
        match value {
            FilterMode::Nearest => Self::Nearest,
            FilterMode::Linear => Self::Linear,
        }
    }
}

#[repr(C)]
#[derive(Serialize, Deserialize)]
pub enum TilemapLayer {
    Texture = 1,
    Algorithm = 1 << 1,
    Physics = 1 << 2,
}

#[derive(Serialize, Deserialize)]
pub struct SerializedTile {
    pub render_chunk_index: usize,
    pub index: UVec2,
    pub texture_index: u32,
    pub color: Vec4,
}

impl SerializedTile {
    pub fn from_tile(tile: &Tile) -> Self {
        Self {
            render_chunk_index: tile.render_chunk_index,
            index: tile.index,
            texture_index: tile.texture_index,
            color: tile.color,
        }
    }
}

#[cfg(feature = "algorithm")]
#[derive(Serialize, Deserialize)]
pub struct SerializedPathTilemap {
    pub tiles: HashMap<UVec2, SerializedPathTile>,
}

#[cfg(feature = "algorithm")]
impl SerializedPathTilemap {
    pub fn from_tilemap(tilemap: &crate::tilemap::algorithm::path::PathTilemap) -> Self {
        let mut tiles = HashMap::default();
        for (index, tile) in tilemap.tiles.iter() {
            tiles.insert(*index, SerializedPathTile::from_path_tile(tile));
        }
        Self { tiles }
    }
}

#[cfg(feature = "algorithm")]
#[derive(Serialize, Deserialize)]
pub struct SerializedPathTile {
    pub cost: u32,
}

#[cfg(feature = "algorithm")]
impl SerializedPathTile {
    pub fn from_path_tile(path_tile: &crate::algorithm::pathfinding::PathTile) -> Self {
        Self {
            cost: path_tile.cost,
        }
    }
}
