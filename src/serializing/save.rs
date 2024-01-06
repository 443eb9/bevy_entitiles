use std::{
    fs::{create_dir_all, File},
    io::Write,
};

use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        system::{Commands, Query},
    },
    math::UVec2,
    reflect::Reflect,
};
use serde::Serialize;

use crate::tilemap::{
    map::{
        TilePivot, TileRenderSize, TilemapAnimations, TilemapLayerOpacities, TilemapName,
        TilemapSlotSize, TilemapStorage, TilemapTexture, TilemapTransform, TilemapType,
    },
    tile::Tile,
};

use super::{pattern::TilemapPattern, SerializedTilemap, TilemapLayer, TILEMAP_META, TILES};

#[cfg(feature = "algorithm")]
use super::PATH_TILES;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum TilemapSaverMode {
    Tilemap,
    MapPattern,
}

pub struct TilemapSaverBuilder {
    path: String,
    texture_path: Option<String>,
    layers: u32,
    remove_map_after_done: bool,
    mode: TilemapSaverMode,
}

impl TilemapSaverBuilder {
    /// For example if path = C:\\maps, then the crate will create:
    /// ```
    /// C
    /// └── maps
    ///     └── (your tilemap's name)
    ///         ├── tilemap.ron
    ///         └── (and other data)
    /// ```
    ///
    /// If the mode is `TilemapSaverMode::MapPattern`, then the crate will create:
    /// ```
    /// C
    /// └── maps
    ///     └── (your tilemap's name).pattern
    /// ```
    pub fn new(path: String) -> Self {
        TilemapSaverBuilder {
            path,
            texture_path: None,
            layers: 0,
            remove_map_after_done: false,
            mode: TilemapSaverMode::Tilemap,
        }
    }

    /// Set which layers to save. By default, only the texture layer is saved.
    /// If there's async algorithms performing when saving, you should save them.
    pub fn with_layer(mut self, layer: TilemapLayer) -> Self {
        self.layers |= layer as u32;
        self
    }

    /// Set the texture path to save.
    pub fn with_texture(mut self, texture_path: String) -> Self {
        self.texture_path = Some(texture_path);
        self
    }

    /// Despawn the tilemap after saving.
    pub fn remove_map_after_done(mut self) -> Self {
        self.remove_map_after_done = true;
        self
    }

    /// Set the saver mode, default is `TilemapSaverMode::Tilemap`.
    pub fn with_mode(mut self, mode: TilemapSaverMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn build(self, commands: &mut Commands, target: Entity) {
        commands.entity(target).insert(TilemapSaver {
            path: self.path,
            texture_path: self.texture_path,
            layers: self.layers,
            remove_map_after_done: self.remove_map_after_done,
            mode: self.mode,
        });
    }
}

#[derive(Component, Reflect)]
pub struct TilemapSaver {
    pub(crate) path: String,
    pub(crate) texture_path: Option<String>,
    pub(crate) layers: u32,
    pub(crate) remove_map_after_done: bool,
    pub(crate) mode: TilemapSaverMode,
}

pub fn save(
    mut commands: Commands,
    tilemaps_query: Query<(
        Entity,
        &TilemapName,
        &TileRenderSize,
        &TilemapSlotSize,
        &TilemapType,
        &TilePivot,
        &TilemapLayerOpacities,
        &TilemapStorage,
        &TilemapTransform,
        Option<&TilemapTexture>,
        Option<&TilemapAnimations>,
        &TilemapSaver,
    )>,
    tiles_query: Query<&Tile>,
    #[cfg(feature = "algorithm")] path_tilemaps_query: Query<
        &crate::tilemap::algorithm::path::PathTilemap,
    >,
) {
    for (
        entity,
        name,
        tile_render_size,
        tile_slot_size,
        ty,
        tile_pivot,
        layer_opacities,
        storage,
        transform,
        texture,
        animations,
        saver,
    ) in tilemaps_query.iter()
    {
        let map_path = format!("{}\\{}\\", saver.path, name.0);

        if saver.mode == TilemapSaverMode::Tilemap {
            let serialized_tilemap = SerializedTilemap::from_tilemap(
                name.clone(),
                *tile_render_size,
                *tile_slot_size,
                *ty,
                *tile_pivot,
                *layer_opacities,
                storage.clone(),
                transform.clone(),
                texture.cloned(),
                animations.cloned(),
                saver,
            );
            save_object(&map_path, TILEMAP_META, &serialized_tilemap);
        }
        let mut pattern = TilemapPattern {
            label: None,
            size: UVec2::ZERO,
            tiles: vec![],
            #[cfg(feature = "algorithm")]
            path_tiles: None,
        };

        // color
        if saver.layers & 1 != 0 {
            let ser_tiles = storage
                .storage
                .chunks
                .values()
                .map(|e| e.iter())
                .flatten()
                .map(|e| {
                    if let Some(tile) = e {
                        Some(tiles_query.get(tile.clone()).cloned().unwrap().into())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            match saver.mode {
                TilemapSaverMode::Tilemap => save_object(&map_path, TILES, &ser_tiles),
                TilemapSaverMode::MapPattern => {
                    pattern.size = storage.storage.size();
                    pattern.tiles = ser_tiles;
                }
            }
        }

        // algorithm
        #[cfg(feature = "algorithm")]
        if saver.layers & (1 << 1) != 0 {
            if let Ok(path_tilemap) = path_tilemaps_query.get(entity) {
                match saver.mode {
                    TilemapSaverMode::Tilemap => save_object(&map_path, PATH_TILES, &path_tilemap),
                    TilemapSaverMode::MapPattern => {
                        pattern.path_tiles = Some(path_tilemap.storage.clone().into_mapper())
                    }
                }
            }
        }

        if saver.mode == TilemapSaverMode::MapPattern {
            save_object(
                format!("{}\\", saver.path).as_str(),
                format!("{}.ron", name.0).as_str(),
                &pattern,
            );
        }

        if saver.remove_map_after_done {
            commands.entity(entity).despawn();
        }

        commands.entity(entity).remove::<TilemapSaver>();
    }
}

fn save_object<T: Serialize>(path: &str, file_name: &str, object: &T) {
    let _ = create_dir_all(path);
    let path = format!("{}{}", path, file_name);
    let _ = File::create(path.clone())
        .unwrap_or(File::open(path).unwrap())
        .write(ron::to_string(object).unwrap().as_bytes());
}
