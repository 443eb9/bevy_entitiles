use std::marker::PhantomData;

use bevy::{
    app::{App, Plugin, Update},
    asset::Handle,
    ecs::entity::Entity,
    render::render_resource::FilterMode,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{
    render::material::TilemapMaterial,
    tilemap::{
        bundles::{MaterialTilemapBundle, StandardPureColorTilemapBundle},
        chunking::storage::ChunkedStorage,
        map::{
            TilePivot, TileRenderSize, TilemapAnimations, TilemapLayerOpacities, TilemapName,
            TilemapSlotSize, TilemapStorage, TilemapTextureDescriptor, TilemapTextures,
            TilemapTransform, TilemapType,
        },
        tile::TileBuilder,
    },
};

use self::save::TilemapSaver;

pub const TILEMAP_META: &str = "tilemap.ron";
pub const TILES: &str = "tiles.ron";
pub const PATH_TILES: &str = "path_tiles.ron";
pub const PHYSICS_TILES: &str = "physics_tiles.ron";

pub mod load;
pub mod save;

#[derive(Default)]
pub struct EntiTilesTilemapSerializingPlugin<M: TilemapMaterial + Serialize + DeserializeOwned>(
    PhantomData<M>,
);

impl<M: TilemapMaterial + Serialize + DeserializeOwned> Plugin
    for EntiTilesTilemapSerializingPlugin<M>
{
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (save::save::<M>, load::load::<M>));
    }
}

#[derive(Serialize, Deserialize)]
pub struct SerializedTilemapData<M: TilemapMaterial> {
    pub tilemap: SerializedTilemap<M>,
    pub tiles: Vec<TileBuilder>,
}

#[derive(Serialize, Deserialize)]
pub struct SerializedTilemap<M: TilemapMaterial> {
    pub name: TilemapName,
    pub tile_render_size: TileRenderSize,
    pub slot_size: TilemapSlotSize,
    pub ty: TilemapType,
    pub tile_pivot: TilePivot,
    pub layer_opacities: TilemapLayerOpacities,
    pub tilemap_transform: TilemapTransform,
    pub material: M,
    pub textures: Option<(Vec<SerializedTilemapTexture>, SerializedFilterMode)>,
    pub animations: Option<TilemapAnimations>,
    pub layers: TilemapLayer,
    pub chunk_size: u32,
}

impl<M: TilemapMaterial> SerializedTilemap<M> {
    pub fn from_tilemap(
        name: TilemapName,
        tile_render_size: TileRenderSize,
        slot_size: TilemapSlotSize,
        ty: TilemapType,
        tile_pivot: TilePivot,
        layer_opacities: TilemapLayerOpacities,
        storage: TilemapStorage,
        tilemap_transform: TilemapTransform,
        texture: Option<TilemapTextures>,
        material: M,
        animations: Option<TilemapAnimations>,
        saver: &TilemapSaver,
    ) -> Self {
        SerializedTilemap {
            name: name.clone(),
            ty,
            tile_render_size,
            slot_size,
            tile_pivot,
            material,
            textures: texture.map(|tex| {
                (
                    tex.textures
                        .into_iter()
                        .zip(saver.texture_path.as_ref().unwrap())
                        .map(|(tex, path)| SerializedTilemapTexture {
                            path: path.clone(),
                            desc: tex.desc.clone(),
                        })
                        .collect(),
                    tex.filter_mode.into(),
                )
            }),
            layer_opacities,
            tilemap_transform,
            layers: saver.layers,
            animations,
            chunk_size: storage.storage.chunk_size,
        }
    }

    pub fn into_tilemap(
        &self,
        tilemap: Entity,
        textures: Handle<TilemapTextures>,
        material: Handle<M>,
    ) -> MaterialTilemapBundle<M> {
        MaterialTilemapBundle {
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
            material,
            transform: self.tilemap_transform,
            textures,
            animations: self.animations.clone().unwrap(),
            ..Default::default()
        }
    }

    pub fn into_pure_color_tilemap(&self, tilemap: Entity) -> StandardPureColorTilemapBundle {
        StandardPureColorTilemapBundle {
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
            transform: self.tilemap_transform,
            ..Default::default()
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SerializedTilemapTexture {
    pub path: String,
    pub desc: TilemapTextureDescriptor,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
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

bitflags::bitflags! {
    #[derive(Serialize, Deserialize, Hash, Eq, PartialEq, Clone, Copy, Debug)]
    pub struct TilemapLayer: u32 {
        const COLOR = 1;
        const PATH = 1 << 1;
        const PHYSICS = 1 << 2;
    }
}
