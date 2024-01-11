use bevy::{
    app::{App, Plugin, Update},
    ecs::entity::Entity,
    math::UVec2,
    reflect::Reflect,
    render::render_resource::FilterMode,
};
use serde::{Deserialize, Serialize};

use crate::tilemap::{
    bundles::{PureColorTilemapBundle, TilemapBundle},
    chunking::storage::ChunkedStorage,
    map::{
        TilePivot, TileRenderSize, TilemapAnimations, TilemapLayerOpacities, TilemapName,
        TilemapRotation, TilemapSlotSize, TilemapStorage, TilemapTexture, TilemapTextureDescriptor,
        TilemapTransform, TilemapType,
    },
    tile::TileBuilder,
};

use self::{
    load::{TilemapLoadFailure, TilemapLoader},
    save::TilemapSaver,
};

pub const TILEMAP_META: &str = "tilemap.ron";
pub const TILES: &str = "tiles.ron";
pub const PATH_TILES: &str = "path_tiles.ron";

pub mod load;
pub mod save;

pub struct EntiTilesTilemapSerializingPlugin;

impl Plugin for EntiTilesTilemapSerializingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (save::save, load::load));

        app.register_type::<TilemapLoader>()
            .register_type::<TilemapSaver>()
            .register_type::<TilemapLoadFailure>()
            .register_type::<SerializedTilemapData>()
            .register_type::<SerializedTilemap>()
            .register_type::<SerializedTilemapTextureDescriptor>()
            .register_type::<SerializedTilemapTexture>();
    }
}

#[derive(Serialize, Deserialize, Reflect)]
pub struct SerializedTilemapData {
    pub tilemap: SerializedTilemap,
    pub tiles: Vec<TileBuilder>,
}

#[derive(Serialize, Deserialize, Reflect)]
pub struct SerializedTilemap {
    pub name: TilemapName,
    pub tile_render_size: TileRenderSize,
    pub slot_size: TilemapSlotSize,
    pub ty: TilemapType,
    pub tile_pivot: TilePivot,
    pub layer_opacities: TilemapLayerOpacities,
    pub tilemap_transform: TilemapTransform,
    pub texture: Option<SerializedTilemapTexture>,
    pub animations: Option<TilemapAnimations>,
    pub layers: u32,
    pub chunk_size: u32,
}

impl SerializedTilemap {
    pub fn from_tilemap(
        name: TilemapName,
        tile_render_size: TileRenderSize,
        slot_size: TilemapSlotSize,
        ty: TilemapType,
        tile_pivot: TilePivot,
        layer_opacities: TilemapLayerOpacities,
        storage: TilemapStorage,
        tilemap_transform: TilemapTransform,
        texture: Option<TilemapTexture>,
        animations: Option<TilemapAnimations>,
        saver: &TilemapSaver,
    ) -> Self {
        SerializedTilemap {
            name: name.clone(),
            ty,
            tile_render_size,
            slot_size,
            tile_pivot,
            texture: texture.and_then(|tex| {
                Some(SerializedTilemapTexture {
                    path: saver.texture_path.clone().unwrap(),
                    desc: tex.desc.into(),
                    rotation: tex.rotation,
                })
            }),
            layer_opacities,
            tilemap_transform,
            layers: saver.layers,
            animations,
            chunk_size: storage.storage.chunk_size,
        }
    }

    pub fn into_tilemap(&self, tilemap: Entity, texture: TilemapTexture) -> TilemapBundle {
        TilemapBundle {
            name: self.name.clone(),
            ty: self.ty,
            tile_render_size: self.tile_render_size,
            slot_size: self.slot_size,
            tile_pivot: self.tile_pivot,
            layer_opacities: self.layer_opacities,
            storage: TilemapStorage {
                tilemap,
                storage: ChunkedStorage::new(self.chunk_size),
                ..Default::default()
            },
            tilemap_transform: self.tilemap_transform,
            texture,
            animations: self.animations.clone().unwrap(),
            ..Default::default()
        }
    }

    pub fn into_pure_color_tilemap(&self, tilemap: Entity) -> PureColorTilemapBundle {
        PureColorTilemapBundle {
            name: self.name.clone(),
            ty: self.ty,
            tile_render_size: self.tile_render_size,
            slot_size: self.slot_size,
            tile_pivot: self.tile_pivot,
            layer_opacities: self.layer_opacities,
            storage: TilemapStorage {
                tilemap,
                storage: ChunkedStorage::new(self.chunk_size),
                ..Default::default()
            },
            tilemap_transform: self.tilemap_transform,
            ..Default::default()
        }
    }
}

#[derive(Serialize, Deserialize, Reflect)]
pub struct SerializedTilemapTexture {
    pub path: String,
    pub desc: SerializedTilemapTextureDescriptor,
    pub rotation: TilemapRotation,
}

#[derive(Serialize, Deserialize, Clone, Reflect)]
pub struct SerializedTilemapTextureDescriptor {
    pub size: UVec2,
    pub tile_size: UVec2,
    pub filter_mode: SerializedFilterMode,
}

impl From<TilemapTextureDescriptor> for SerializedTilemapTextureDescriptor {
    fn from(value: TilemapTextureDescriptor) -> Self {
        Self {
            size: value.size,
            tile_size: value.tile_size,
            filter_mode: value.filter_mode.into(),
        }
    }
}

impl Into<TilemapTextureDescriptor> for SerializedTilemapTextureDescriptor {
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
#[derive(Serialize, Deserialize, Reflect, Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub enum TilemapLayer {
    Color = 1,
    Path = 1 << 1,
    // TODO
    // Physics = 1 << 2,
}
