use std::path::Path;

use bevy::{
    asset::{Assets, Handle},
    ecs::{
        component::Component,
        entity::Entity,
        system::{Commands, Query, Res},
    },
    reflect::Reflect,
};
use serde::Serialize;

use crate::{
    render::material::TilemapMaterial,
    serializing::map::{SerializedTilemap, TilemapLayer, TILEMAP_META, TILES},
    serializing::{pattern::TilemapPattern, save_object},
    tilemap::{
        chunking::storage::ChunkedStorage,
        despawn::DespawnMe,
        map::{
            TilePivot, TileRenderSize, TilemapAnimations, TilemapLayerOpacities, TilemapName,
            TilemapSlotSize, TilemapStorage, TilemapTextures, TilemapTransform, TilemapType,
        },
        tile::{Tile, TileBuilder},
    },
};

#[cfg(feature = "algorithm")]
use crate::{algorithm::pathfinding::PathTilemaps, serializing::map::PATH_TILES};

#[cfg(feature = "physics")]
use crate::{
    serializing::map::PHYSICS_TILES,
    tilemap::{buffers::PackedPhysicsTileBuffer, physics::SerializablePhysicsSource},
};

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
    pub texture_path: Option<Vec<String>>,
    pub remove_after_save: bool,
}

pub fn save<M: TilemapMaterial + Serialize>(
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
        &Handle<M>,
        Option<&Handle<TilemapTextures>>,
        Option<&TilemapAnimations>,
        &TilemapSaver,
    )>,
    tiles_query: Query<&Tile>,
    textures_assets: Res<Assets<TilemapTextures>>,
    material_assets: Res<Assets<M>>,
    #[cfg(feature = "algorithm")] path_tilemaps: Res<PathTilemaps>,
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
        material,
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
                texture.and_then(|t| textures_assets.get(t)).cloned(),
                material_assets.get(material).unwrap().clone(),
                animations.cloned(),
                saver,
            );
            save_object(&map_path, TILEMAP_META, &serialized_tilemap);
        }
        let mut pattern = TilemapPattern::new(Some(name.0.clone()));

        // color
        if saver.layers.contains(TilemapLayer::COLOR) {
            let chunk_size = storage.storage.chunk_size;
            let ser_tiles = storage.storage.chunked_iter_some().fold(
                ChunkedStorage::<TileBuilder>::new(chunk_size),
                |mut acc, (chunk_index, in_chunk_index, tile)| {
                    acc.set_elem_precise(
                        chunk_index,
                        in_chunk_index,
                        tiles_query.get(*tile).unwrap().clone().into(),
                    );
                    acc
                },
            );

            match saver.mode {
                TilemapSaverMode::Tilemap => save_object(&map_path, TILES, &ser_tiles),
                TilemapSaverMode::MapPattern => {
                    pattern.tiles.tiles = ser_tiles.into_mapper();
                    pattern.tiles.recalculate_rect();
                }
            }
        }

        // algorithm
        #[cfg(feature = "algorithm")]
        if saver.layers.contains(TilemapLayer::PATH) {
            loop {
                #[cfg(feature = "multi-threaded")]
                let Some(path_tilemap) = path_tilemaps.lock(entity) else {
                    break;
                };
                #[cfg(not(feature = "multi-threaded"))]
                let Some(path_tilemap) = path_tilemaps.get(entity) else {
                    break;
                };

                match saver.mode {
                    TilemapSaverMode::Tilemap => {
                        save_object(&map_path, PATH_TILES, &path_tilemap.storage)
                    }
                    TilemapSaverMode::MapPattern => {
                        pattern.path_tiles.tiles = path_tilemap.storage.clone().into_mapper();
                        pattern.path_tiles.recalculate_rect();
                    }
                }
                break;
            }
        }

        #[cfg(feature = "physics")]
        if saver.layers.contains(TilemapLayer::PHYSICS) {
            if let Ok(physics_tilemap) = physics_tilemaps_query.get(entity) {
                match saver.mode {
                    TilemapSaverMode::Tilemap => {
                        save_object(&map_path, PHYSICS_TILES, &physics_tilemap.data)
                    }
                    TilemapSaverMode::MapPattern => {
                        let mut buffer = PackedPhysicsTileBuffer::new();
                        buffer.tiles = physics_tilemap
                            .data
                            .clone()
                            .into_mapper()
                            .into_iter()
                            .map(|(index, mut tile)| {
                                tile.collider.as_verts_mut().iter_mut().for_each(|v| {
                                    *v = *v - transform.translation;
                                });
                                (index, tile)
                            })
                            .collect();
                        buffer.recalculate_rect();
                        pattern.physics_tiles = SerializablePhysicsSource::Buffer(buffer);
                    }
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
