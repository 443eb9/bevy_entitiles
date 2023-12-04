use bevy::{
    app::{Plugin, Update},
    ecs::entity::Entity,
    math::{UVec2, Vec2, Vec4},
    render::render_resource::FilterMode,
    utils::HashMap,
};
use serde::{Deserialize, Serialize};

use crate::{
    math::aabb::AabbBox2d,
    render::texture::{TileUV, TilemapTextureDescriptor, TilemapTexture},
    tilemap::{
        map::Tilemap,
        tile::{Tile, TileAnimation, TileType},
    },
};

use self::{
    load::load,
    save::{save, TilemapSaver},
};

pub const TILEMAP_META: &str = "tilemap.ron";
pub const TILES: &str = "tiles.ron";
pub const PATH_TILES: &str = "path_tiles.ron";

pub mod load;
pub mod save;

pub struct EntiTilesSerializingPlugin;

impl Plugin for EntiTilesSerializingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Update, (save, load));
    }
}

#[derive(Serialize, Deserialize)]
pub struct SerializedTilemapData {
    pub tilemap: SerializedTilemap,
    pub tiles: Vec<SerializedTile>,
}

#[derive(Serialize, Deserialize)]
pub struct SerializedTilemap {
    pub name: String,
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
    pub z_order: f32,
    pub layers: u32,
}

impl SerializedTilemap {
    pub fn from_tilemap(tilemap: &Tilemap, saver: &TilemapSaver) -> Self {
        SerializedTilemap {
            name: tilemap.name.clone(),
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
            layers: saver.layers,
        }
    }

    pub fn into_tilemap(
        &self,
        entity: Entity,
        texture: Option<TilemapTexture>,
    ) -> Tilemap {
        Tilemap {
            id: entity,
            name: self.name.clone(),
            tile_type: self.tile_type,
            size: self.size,
            tile_render_scale: self.tile_render_scale,
            tile_slot_size: self.tile_slot_size,
            anchor: self.anchor,
            render_chunk_size: self.render_chunk_size,
            texture,
            tiles: vec![],
            flip: self.flip,
            aabb: self.aabb,
            translation: self.translation,
            z_order: self.z_order,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SerializedTilemapTexture {
    pub path: String,
    pub desc: SerializedTilemapDescriptor,
}

#[derive(Serialize, Deserialize, Clone)]
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

impl Into<TilemapTextureDescriptor> for SerializedTilemapDescriptor {
    fn into(self) -> TilemapTextureDescriptor {
        TilemapTextureDescriptor {
            size: self.size,
            tiles_uv: self.tiles_uv,
            filter_mode: self.filter_mode.into(),
            is_uniform: self.is_uniform,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
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

impl Into<FilterMode> for SerializedFilterMode {
    fn into(self) -> FilterMode {
        match self {
            Self::Nearest => FilterMode::Nearest,
            Self::Linear => FilterMode::Linear,
        }
    }
}

#[repr(u32)]
#[derive(Serialize, Deserialize)]
pub enum TilemapLayer {
    Texture = 1,
    Algorithm = 1 << 1,
    Physics = 1 << 2,
    All = !0,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SerializedTile {
    pub index: UVec2,
    pub texture_index: u32,
    pub color: Vec4,
    pub anim: Option<TileAnimation>,
}

impl SerializedTile {
    fn from_tile(tile: Tile, anim: Option<TileAnimation>) -> Self {
        Self {
            index: tile.index,
            texture_index: tile.texture_index,
            color: tile.color,
            anim,
        }
    }
}

#[cfg(feature = "algorithm")]
#[derive(Serialize, Deserialize)]
pub struct SerializedPathTilemap {
    pub tiles: HashMap<UVec2, SerializedPathTile>,
}

#[cfg(feature = "algorithm")]
impl From<crate::tilemap::algorithm::path::PathTilemap> for SerializedPathTilemap {
    fn from(value: crate::tilemap::algorithm::path::PathTilemap) -> Self {
        let mut tiles = HashMap::default();
        for (index, tile) in value.tiles.iter() {
            tiles.insert(*index, (*tile).into());
        }
        Self { tiles }
    }
}

#[cfg(feature = "algorithm")]
impl SerializedPathTilemap {
    fn into_path_tilemap(self, tilemap: Entity) -> crate::tilemap::algorithm::path::PathTilemap {
        let mut tiles = HashMap::default();
        for (index, tile) in self.tiles.iter() {
            tiles.insert(*index, tile.clone().into());
        }
        crate::tilemap::algorithm::path::PathTilemap { tilemap, tiles }
    }
}

#[cfg(feature = "algorithm")]
#[derive(Serialize, Deserialize, Clone)]
pub struct SerializedPathTile {
    pub cost: u32,
}

#[cfg(feature = "algorithm")]
impl From<crate::algorithm::pathfinding::PathTile> for SerializedPathTile {
    fn from(value: crate::algorithm::pathfinding::PathTile) -> Self {
        Self { cost: value.cost }
    }
}

#[cfg(feature = "algorithm")]
impl Into<crate::algorithm::pathfinding::PathTile> for SerializedPathTile {
    fn into(self) -> crate::algorithm::pathfinding::PathTile {
        crate::algorithm::pathfinding::PathTile { cost: self.cost }
    }
}
