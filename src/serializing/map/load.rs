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
    reflect::Reflect,
    utils::HashMap,
};

use crate::{
    serializing::load_object,
    tilemap::{
        map::{TilemapStorage, TilemapTexture},
        storage::ChunkedStorage,
        tile::TileBuilder,
    },
};

use super::{SerializedTilemap, TilemapLayer, TILEMAP_META, TILES};

#[cfg(feature = "algorithm")]
use super::PATH_TILES;

pub struct TilemapLoaderBuilder {
    path: String,
    map_name: String,
    layers: u32,
}

impl TilemapLoaderBuilder {
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
    pub fn new(path: String, map_name: String) -> Self {
        TilemapLoaderBuilder {
            path,
            map_name,
            layers: 0,
        }
    }

    pub fn with_layer(mut self, layer: TilemapLayer) -> Self {
        self.layers |= layer as u32;
        self
    }

    pub fn build(self, commands: &mut Commands, target: Entity) {
        commands.entity(target).insert(TilemapLoader {
            path: self.path,
            map_name: self.map_name,
            layers: self.layers,
        });
    }
}

#[derive(Component, Clone, Reflect)]
pub struct TilemapLoader {
    pub(crate) path: String,
    pub(crate) map_name: String,
    pub(crate) layers: u32,
}

#[derive(Component, Reflect)]
pub struct TilemapLoadFailure {
    pub path: String,
    pub map_name: String,
    pub layers: u32,
}

impl From<TilemapLoader> for TilemapLoadFailure {
    fn from(loader: TilemapLoader) -> Self {
        TilemapLoadFailure {
            path: loader.path,
            map_name: loader.map_name,
            layers: loader.layers,
        }
    }
}

pub fn load(
    mut commands: Commands,
    tilemaps_query: Query<(Entity, &TilemapLoader)>,
    asset_server: Res<AssetServer>,
) {
    for (entity, loader) in tilemaps_query.iter() {
        let map_path = Path::new(&loader.path).join(&loader.map_name);
        let failure = <TilemapLoader as Into<TilemapLoadFailure>>::into(loader.clone());

        let Ok(ser_tilemap) = load_object::<SerializedTilemap>(&map_path, TILEMAP_META) else {
            complete(&mut commands, entity, failure, false);
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
        let ser_tiles = if loader.layers & 1 != 0 && ser_tilemap.layers & 1 != 0 {
            Some(load_object::<HashMap<IVec2, TileBuilder>>(&map_path, TILES))
        } else {
            None
        };

        // algorithm
        #[cfg(feature = "algorithm")]
        if loader.layers & (1 << 1) != 0 {
            let Ok(path_tilemap) =
                load_object::<crate::tilemap::algorithm::path::PathTilemap>(&map_path, PATH_TILES)
            else {
                complete(&mut commands, entity, failure, false);
                continue;
            };

            commands.entity(entity).insert(path_tilemap);
        }

        let mut storage = TilemapStorage {
            tilemap: entity,
            storage: ChunkedStorage::new(ser_tilemap.chunk_size),
        };

        if let Some(ser_tiles) = ser_tiles {
            let Ok(ser_tiles) = ser_tiles else {
                complete(&mut commands, entity, failure, false);
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
