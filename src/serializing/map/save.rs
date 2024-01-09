use std::path::Path;

use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        system::{Commands, Query},
    },
    reflect::Reflect,
    utils::HashMap,
};

use crate::{
    serializing::save_object,
    tilemap::{
        map::{
            TilePivot, TileRenderSize, TilemapAnimations, TilemapLayerOpacities, TilemapName,
            TilemapSlotSize, TilemapStorage, TilemapTexture, TilemapTransform, TilemapType,
        },
        tile::{Tile, TileBuilder},
    },
};

use super::{pattern::TilemapPattern, SerializedTilemap, TilemapLayer, TILEMAP_META, TILES};

#[cfg(feature = "algorithm")]
use super::PATH_TILES;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum TilemapSaverMode {
    Tilemap,
    MapPattern,
}

#[derive(Component, Reflect)]
pub struct TilemapSaver {
    pub(crate) path: String,
    pub(crate) texture_path: Option<String>,
    pub(crate) layers: u32,
    pub(crate) remove_after_save: bool,
    pub(crate) mode: TilemapSaverMode,
}

impl TilemapSaver {
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
        TilemapSaver {
            path,
            texture_path: None,
            layers: 0,
            remove_after_save: false,
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
    pub fn remove_after_save(mut self) -> Self {
        self.remove_after_save = true;
        self
    }

    /// Set the saver mode, default is `TilemapSaverMode::Tilemap`.
    pub fn with_mode(mut self, mode: TilemapSaverMode) -> Self {
        self.mode = mode;
        self
    }
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
        slot_size,
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
        let map_dir = Path::new(&saver.path);
        let map_path = map_dir.join(&name.0);

        if saver.mode == TilemapSaverMode::Tilemap {
            let serialized_tilemap = SerializedTilemap::from_tilemap(
                name.clone(),
                *tile_render_size,
                *slot_size,
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
        let mut pattern = TilemapPattern::new(Some(name.0.clone()));

        // color
        if saver.layers & 1 != 0 {
            let ser_tiles = storage
                .storage
                .clone()
                .into_mapper()
                .iter()
                .map(|t| {
                    (
                        *t.0,
                        <Tile as Into<TileBuilder>>::into(tiles_query.get(*t.1).unwrap().clone()),
                    )
                })
                .collect::<HashMap<_, _>>();

            match saver.mode {
                TilemapSaverMode::Tilemap => save_object(&map_path, TILES, &ser_tiles),
                TilemapSaverMode::MapPattern => {
                    pattern.tiles.tiles = ser_tiles;
                    pattern.tiles.recalculate_aabb();
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
                        pattern.path_tiles.tiles = path_tilemap.storage.clone().into_mapper();
                        pattern.path_tiles.recalculate_aabb();
                    }
                }
            }
        }

        if saver.mode == TilemapSaverMode::MapPattern {
            save_object(map_dir, format!("{}.ron", name.0).as_str(), &pattern);
        }

        if saver.remove_after_save {
            commands.entity(entity).despawn();
        }

        commands.entity(entity).remove::<TilemapSaver>();
    }
}
