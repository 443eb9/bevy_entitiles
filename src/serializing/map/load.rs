use std::path::Path;

use bevy::{
    asset::AssetServer,
    ecs::{
        bundle::Bundle,
        component::Component,
        entity::Entity,
        system::{Commands, Query, Res},
    },
    hierarchy::DespawnRecursiveExt,
    math::IVec2,
    utils::HashMap,
};

use crate::{
    serializing::load_object,
    tilemap::{
        chunking::storage::ChunkedStorage,
        map::{TilemapStorage, TilemapTexture},
        tile::TileBuilder,
    },
};

use super::{SerializedTilemap, TilemapLayer, TILEMAP_META, TILES};

#[cfg(feature = "algorithm")]
use crate::{serializing::map::PATH_TILES, tilemap::algorithm::path::PathTilemap};

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

pub fn load(
    mut commands: Commands,
    tilemaps_query: Query<(Entity, &TilemapLoader)>,
    asset_server: Res<AssetServer>,
) {
    for (entity, loader) in tilemaps_query.iter() {
        let map_path = Path::new(&loader.path).join(&loader.map_name);

        let Ok(ser_tilemap) = load_object::<SerializedTilemap>(&map_path, TILEMAP_META) else {
            complete(&mut commands, entity, (), false);
            continue;
        };

        let texture = if let Some(tex) = &ser_tilemap.texture {
            Some(TilemapTexture {
                texture: asset_server.load(tex.path.clone()),
                desc: tex.desc.clone().into(),
                rotation: tex.rotation,
            })
        } else {
            None
        };

        // texture
        let ser_tiles = if loader.layers.contains(TilemapLayer::COLOR) {
            Some(load_object::<HashMap<IVec2, TileBuilder>>(&map_path, TILES))
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

            ser_tiles.into_iter().for_each(|(index, tile)| {
                storage.set(&mut commands, index, tile.into());
            });
        }

        if let Some(tex) = texture {
            let mut bundle = ser_tilemap.into_tilemap(entity, tex);
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
            let Ok(path_tilemap) = load_object::<PathTilemap>(&map_path, PATH_TILES) else {
                complete(&mut commands, entity, (), false);
                continue;
            };

            commands.entity(entity).insert(path_tilemap);
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
                        tile.spawn(&mut commands, ser_tilemap.ty),
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
