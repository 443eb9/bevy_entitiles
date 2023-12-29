use bevy::{
    app::{Plugin, Update},
    ecs::entity::Entity,
    math::{UVec2, Vec2, Vec4},
    reflect::Reflect,
    render::render_resource::FilterMode,
};
use serde::{Deserialize, Serialize};

#[cfg(feature = "algorithm")]
use bevy::utils::HashMap;

use crate::{
    math::aabb::Aabb2d,
    reflect::ReflectFilterMode,
    render::{
        buffer::TileAnimation,
        texture::{TilemapTexture, TilemapTextureDescriptor},
    },
    tilemap::{
        map::{Tilemap, TilemapRotation, TilemapTransform},
        tile::{Tile, TileTexture, TileType},
    },
    MAX_ANIM_COUNT,
};

use self::{
    load::{load, TilemapLoadFailure, TilemapLoader},
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

        app.register_type::<TilemapLoader>()
            .register_type::<TilemapSaver>()
            .register_type::<TilemapLoadFailure>()
            .register_type::<SerializedTilemapData>()
            .register_type::<SerializedTilemap>()
            .register_type::<SerializedTilemapDescriptor>()
            .register_type::<SerializedTilemapTexture>();

        #[cfg(feature = "algorithm")]
        app.register_type::<SerializedPathTile>()
            .register_type::<SerializedPathTilemap>();
    }
}

#[derive(Serialize, Deserialize, Reflect)]
pub struct SerializedTilemapData {
    pub tilemap: SerializedTilemap,
    pub tiles: Vec<SerializedTile>,
}

#[derive(Serialize, Deserialize, Reflect)]
pub struct SerializedTilemap {
    pub name: String,
    pub tile_type: TileType,
    pub ext_dir: Vec2,
    pub size: UVec2,
    pub tile_render_size: Vec2,
    pub tile_slot_size: Vec2,
    pub pivot: Vec2,
    pub render_chunk_size: u32,
    pub texture: Option<SerializedTilemapTexture>,
    pub layer_opacities: Vec4,
    pub aabb: Aabb2d,
    pub transform: TilemapTransform,
    pub layers: u32,
    pub anim_seqs: Vec<TileAnimation>,
    pub anim_counts: usize,
}

impl SerializedTilemap {
    pub fn from_tilemap(tilemap: &Tilemap, saver: &TilemapSaver) -> Self {
        SerializedTilemap {
            name: tilemap.name.clone(),
            tile_type: tilemap.tile_type,
            ext_dir: tilemap.ext_dir,
            size: tilemap.size,
            tile_render_size: tilemap.tile_render_size,
            tile_slot_size: tilemap.tile_slot_size,
            pivot: tilemap.pivot,
            render_chunk_size: tilemap.render_chunk_size,
            texture: if let Some(tex) = &saver.texture_path {
                Some(SerializedTilemapTexture {
                    path: tex.clone(),
                    desc: tilemap.texture.as_ref().unwrap().desc.clone().into(),
                    rotation: tilemap.texture.as_ref().unwrap().rotation,
                })
            } else {
                None
            },
            layer_opacities: tilemap.layer_opacities,
            aabb: tilemap.aabb,
            transform: tilemap.transform,
            layers: saver.layers,
            anim_seqs: tilemap.anim_seqs.to_vec(),
            anim_counts: tilemap.anim_counts,
        }
    }

    pub fn into_tilemap(&self, entity: Entity, texture: Option<TilemapTexture>) -> Tilemap {
        let mut anim_seqs = [TileAnimation::default(); MAX_ANIM_COUNT];
        for (i, anim) in self.anim_seqs.iter().enumerate() {
            anim_seqs[i] = *anim;
        }
        Tilemap {
            id: entity,
            name: self.name.clone(),
            tile_type: self.tile_type,
            ext_dir: self.ext_dir,
            size: self.size,
            tile_render_size: self.tile_render_size,
            tile_slot_size: self.tile_slot_size,
            pivot: self.pivot,
            render_chunk_size: self.render_chunk_size,
            texture,
            layer_opacities: self.layer_opacities,
            tiles: vec![],
            aabb: self.aabb,
            transform: self.transform,
            anim_seqs,
            anim_counts: self.anim_counts,
        }
    }
}

#[derive(Serialize, Deserialize, Reflect)]
pub struct SerializedTilemapTexture {
    pub path: String,
    pub desc: SerializedTilemapDescriptor,
    pub rotation: TilemapRotation,
}

#[derive(Serialize, Deserialize, Clone, Reflect)]
pub struct SerializedTilemapDescriptor {
    pub size: UVec2,
    pub tile_size: UVec2,
    pub filter_mode: ReflectFilterMode,
}

impl From<TilemapTextureDescriptor> for SerializedTilemapDescriptor {
    fn from(value: TilemapTextureDescriptor) -> Self {
        Self {
            size: value.size,
            tile_size: value.tile_size,
            filter_mode: value.filter_mode.into(),
        }
    }
}

impl Into<TilemapTextureDescriptor> for SerializedTilemapDescriptor {
    fn into(self) -> TilemapTextureDescriptor {
        TilemapTextureDescriptor {
            size: self.size,
            tile_size: self.tile_size,
            filter_mode: self.filter_mode.into(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Reflect)]
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
#[derive(Serialize, Deserialize, Reflect)]
pub enum TilemapLayer {
    Texture = 1,
    Algorithm = 1 << 1,
    Physics = 1 << 2,
    All = !0,
}

#[derive(Serialize, Deserialize, Clone, Reflect)]
pub struct SerializedTile {
    pub index: UVec2,
    pub texture: TileTexture,
    pub color: Vec4,
}

impl SerializedTile {
    fn from_tile(tile: Tile) -> Self {
        Self {
            index: tile.index,
            texture: tile.texture,
            color: tile.color,
        }
    }
}

#[cfg(feature = "algorithm")]
#[derive(Serialize, Deserialize, Reflect)]
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
    fn into_path_tilemap(self) -> crate::tilemap::algorithm::path::PathTilemap {
        let mut tiles = HashMap::default();
        for (index, tile) in self.tiles.iter() {
            tiles.insert(*index, tile.clone().into());
        }
        crate::tilemap::algorithm::path::PathTilemap { tiles }
    }
}

#[cfg(feature = "algorithm")]
#[derive(Serialize, Deserialize, Clone, Reflect)]
pub struct SerializedPathTile {
    pub cost: u32,
}

#[cfg(feature = "algorithm")]
impl From<crate::tilemap::algorithm::path::PathTile> for SerializedPathTile {
    fn from(value: crate::tilemap::algorithm::path::PathTile) -> Self {
        Self { cost: value.cost }
    }
}

#[cfg(feature = "algorithm")]
impl Into<crate::tilemap::algorithm::path::PathTile> for SerializedPathTile {
    fn into(self) -> crate::tilemap::algorithm::path::PathTile {
        crate::tilemap::algorithm::path::PathTile { cost: self.cost }
    }
}
