use std::path::Path;

use bevy::{
    asset::{AssetServer, Assets},
    ecs::{
        bundle::Bundle,
        component::Component,
        entity::Entity,
        system::{Commands, Query, Res, ResMut},
    },
    hierarchy::DespawnRecursiveExt,
};
use serde::de::DeserializeOwned;

use crate::{
    render::material::TilemapMaterial,
    serializing::{
        load_object,
        map::{SerializedTilemap, TilemapLayer, TILEMAP_META, TILES},
    },
    tilemap::{
        chunking::storage::{ChunkedStorage, TileBuilderChunkedStorage},
        map::{TilemapStorage, TilemapTexture, TilemapTextures},
        tile::Tile,
    },
};

#[cfg(feature = "algorithm")]
use crate::{
    algorithm::pathfinding::PathTilemaps,
    serializing::map::PATH_TILES,
    tilemap::{algorithm::path::PathTilemap, chunking::storage::PathTileChunkedStorage},
};

#[cfg(feature = "physics")]
use crate::{
    serializing::map::PHYSICS_TILES,
    tilemap::{chunking::storage::PackedPhysicsTileChunkedStorage, physics::PhysicsTilemap},
};

#[derive(Component, Clone)]
pub struct TilemapLoader {
    /// For example if the file tree look like:
    ///
    /// ```
    /// C
    /// └── maps
    ///     └── beautiful map
    ///         ├── tilemap.ron
    ///         └── (and other data)
    /// ```
    /// Then path = `C:\\maps` and map_name = `beautiful map`
    pub path: String,
    pub map_name: String,
    pub layers: TilemapLayer,
}

pub fn load<M: TilemapMaterial + DeserializeOwned>(
    mut commands: Commands,
    tilemaps_query: Query<(Entity, &TilemapLoader)>,
    asset_server: Res<AssetServer>,
    mut textures_assets: ResMut<Assets<TilemapTextures>>,
    mut material_assets: ResMut<Assets<M>>,
    #[cfg(feature = "algorithm")] mut path_tilemaps: ResMut<PathTilemaps>,
) {
    for (entity, loader) in tilemaps_query.iter() {
        let map_path = Path::new(&loader.path).join(&loader.map_name);

        let Ok(ser_tilemap) = load_object::<SerializedTilemap<M>>(&map_path, TILEMAP_META) else {
            complete(&mut commands, entity, (), false);
            continue;
        };

        let texture = {
            if let Some((tex, filter_mode)) = &ser_tilemap.textures {
                Some(TilemapTextures::new(
                    tex.iter()
                        .map(|tex| TilemapTexture {
                            texture: asset_server.load(&tex.path),
                            desc: tex.desc.clone().into(),
                        })
                        .collect(),
                    (*filter_mode).into(),
                ))
            } else {
                None
            }
        };

        // texture
        let ser_tiles = if loader.layers.contains(TilemapLayer::COLOR) {
            Some(load_object::<TileBuilderChunkedStorage>(&map_path, TILES))
        } else {
            None
        };

        let mut storage = TilemapStorage {
            tilemap: entity,
            storage: ChunkedStorage::new(ser_tilemap.chunk_size),
            ..Default::default()
        };

        // color
        if let Some(ser_tiles) = ser_tiles {
            let Ok(ser_tiles) = ser_tiles else {
                complete(&mut commands, entity, (), false);
                continue;
            };

            let mut bundles = Vec::new();
            ser_tiles
                .chunked_iter_some()
                .for_each(|(chunk_index, in_chunk_index, tile)| {
                    let tile_entity = commands.spawn_empty().id();
                    storage
                        .storage
                        .set_elem_precise(chunk_index, in_chunk_index, tile_entity);
                    bundles.push((
                        tile_entity,
                        Tile {
                            tilemap_id: entity,
                            chunk_index,
                            in_chunk_index,
                            index: storage
                                .storage
                                .inverse_transform_index(chunk_index, in_chunk_index),
                            texture: tile.texture.clone(),
                            tint: tile.tint,
                        },
                    ));
                });
            commands.insert_or_spawn_batch(bundles);
        }

        if let Some(tex) = texture {
            let mut bundle = ser_tilemap.into_tilemap(
                entity,
                textures_assets.add(tex),
                material_assets.add(ser_tilemap.material.clone()),
            );
            bundle.storage = storage;
            complete(&mut commands, entity, bundle, true);
        } else {
            let mut bundle = ser_tilemap.into_pure_color_tilemap(entity);
            bundle.storage = storage;
            complete(&mut commands, entity, bundle, true);
        }

        // algorithm
        #[cfg(feature = "algorithm")]
        if loader.layers.contains(TilemapLayer::PATH) {
            let Ok(path_storage) = load_object::<PathTileChunkedStorage>(&map_path, PATH_TILES)
            else {
                complete(&mut commands, entity, (), false);
                continue;
            };

            path_tilemaps.insert(
                entity,
                PathTilemap {
                    storage: path_storage,
                },
            );
        }

        // physics
        #[cfg(feature = "physics")]
        if loader.layers.contains(TilemapLayer::PHYSICS) {
            let Ok(physics_tiles) =
                load_object::<PackedPhysicsTileChunkedStorage>(&map_path, PHYSICS_TILES)
            else {
                complete(&mut commands, entity, (), false);
                continue;
            };

            let mut physics_storage = ChunkedStorage::new(ser_tilemap.chunk_size);

            physics_tiles
                .chunked_iter_some()
                .for_each(|(chunk_index, in_chunk_index, tile)| {
                    physics_storage.set_elem_precise(
                        chunk_index,
                        in_chunk_index,
                        tile.spawn(&mut commands),
                    );
                });

            commands.entity(entity).insert(PhysicsTilemap {
                storage: physics_storage,
                spawn_queue: Vec::new(),
                data: physics_tiles,
            });
        }
    }
}

fn complete(commands: &mut Commands, entity: Entity, bundle: impl Bundle, is_success: bool) {
    if is_success {
        commands.entity(entity).remove::<TilemapLoader>();
        commands.entity(entity).insert(bundle);
    } else {
        commands.entity(entity).despawn_recursive();
    }
}
