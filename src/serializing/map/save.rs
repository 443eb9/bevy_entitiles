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
    serializing::{pattern::TilemapPattern, save_object},
    tilemap::{
        despawn::DespawnMe,
        map::{
            TilePivot, TileRenderSize, TilemapAnimations, TilemapLayerOpacities, TilemapName,
            TilemapSlotSize, TilemapStorage, TilemapTexture, TilemapTransform, TilemapType,
        },
        tile::{Tile, TileBuilder},
    },
};

use super::{SerializedTilemap, TilemapLayer, PHYSICS_TILES, TILEMAP_META, TILES};

#[cfg(feature = "algorithm")]
use super::PATH_TILES;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum TilemapSaverMode {
    Tilemap,
    MapPattern,
}

#[derive(Component)]
pub struct TilemapSaver {
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
    pub path: String,
    pub mode: TilemapSaverMode,
    pub layers: TilemapLayer,
    pub texture_path: Option<String>,
    pub remove_after_save: bool,
}

pub fn save(
    mut commands: Commands,
    mut tilemaps_query: Query<(
        Entity,
        &TilemapName,
        &TileRenderSize,
        &TilemapSlotSize,
        &TilemapType,
        &TilePivot,
        &TilemapLayerOpacities,
        &mut TilemapStorage,
        &TilemapTransform,
        Option<&TilemapTexture>,
        Option<&TilemapAnimations>,
        &TilemapSaver,
    )>,
    tiles_query: Query<&Tile>,
    #[cfg(feature = "algorithm")] path_tilemaps_query: Query<
        &crate::tilemap::algorithm::path::PathTilemap,
    >,
    #[cfg(feature = "physics")] physics_tilemaps_query: Query<
        &crate::tilemap::physics::PhysicsTilemap,
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
        mut storage,
        transform,
        texture,
        animations,
        saver,
    ) in tilemaps_query.iter_mut()
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
        if saver.layers.contains(TilemapLayer::COLOR) {
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
        if saver.layers.contains(TilemapLayer::PATH) {
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

        #[cfg(feature = "physics")]
        if saver.layers.contains(TilemapLayer::PHYSICS) {
            if let Ok(physics_tilemap) = physics_tilemaps_query.get(entity) {
                match saver.mode {
                    TilemapSaverMode::Tilemap => {
                        save_object(&map_path, PHYSICS_TILES, &physics_tilemap.data)
                    }
                    TilemapSaverMode::MapPattern => todo!(),
                }
            }
        }

        if saver.mode == TilemapSaverMode::MapPattern {
            save_object(map_dir, format!("{}.ron", name.0).as_str(), &pattern);
        }

        if saver.remove_after_save {
            storage.despawn(&mut commands);
            commands.entity(entity).insert(DespawnMe);
        }

        commands.entity(entity).remove::<TilemapSaver>();
    }
}
