use std::fs::read_to_string;

use bevy::{
    asset::AssetServer,
    ecs::{
        component::Component,
        entity::Entity,
        query::Without,
        system::{Commands, Query, Res},
    },
};
use ron::{de::from_bytes, error::SpannedError};
use serde::Deserialize;

use crate::{tilemap::{
    algorithm::path::PathTilemap,
    map::Tilemap,
    tile::TileBuilder,
}, render::texture::TilemapTexture};

use super::{SerializedTile, SerializedTilemap, TilemapLayer, PATH_TILES, TILEMAP_META, TILES};

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

#[derive(Component, Clone)]
pub struct TilemapLoader {
    pub(crate) path: String,
    pub(crate) map_name: String,
    pub(crate) layers: u32,
}

#[derive(Component)]
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
    tilemaps_query: Query<(Entity, &TilemapLoader), (Without<Tilemap>, Without<PathTilemap>)>,
    asset_server: Res<AssetServer>,
) {
    for (entity, loader) in tilemaps_query.iter() {
        let map_path = format!("{}\\{}\\", loader.path, loader.map_name);

        let Ok(serialized_tilemap) = load_object::<SerializedTilemap>(&map_path, TILEMAP_META)
        else {
            complete::<TilemapLoadFailure>(&mut commands, entity, loader.clone().into());
            continue;
        };

        let texture = if let Some(tex) = &serialized_tilemap.texture {
            Some(TilemapTexture {
                texture: asset_server.load(tex.path.clone()),
                desc: tex.desc.clone().into(),
            })
        } else {
            None
        };

        // texture
        let serialized_tiles = if loader.layers & 1 != 0 && serialized_tilemap.layers & 1 != 0 {
            Some(load_object::<Vec<Option<SerializedTile>>>(&map_path, TILES))
        } else {
            None
        };

        // algorithm
        #[cfg(feature = "algorithm")]
        if loader.layers & (1 << 1) != 0 {
            let Ok(serialized_path_tilemap) =
                load_object::<super::SerializedPathTilemap>(&map_path, PATH_TILES)
            else {
                complete::<TilemapLoadFailure>(&mut commands, entity, loader.clone().into());
                continue;
            };

            commands
                .entity(entity)
                .insert(serialized_path_tilemap.into_path_tilemap(entity));
        }

        let mut tilemap = serialized_tilemap.into_tilemap(entity, texture);

        if let Some(ser_tiles) = serialized_tiles {
            let Ok(ser_tiles) = ser_tiles else {
                complete::<TilemapLoadFailure>(&mut commands, entity, loader.clone().into());
                continue;
            };

            tilemap.tiles = vec![None; ser_tiles.len()];
            for i in 0..ser_tiles.len() {
                if let Some(ser_t) = &ser_tiles[i] {
                    tilemap.set(
                        &mut commands,
                        ser_t.index,
                        &TileBuilder::from_serialized_tile(ser_t),
                    );
                }
            }
        }

        complete(&mut commands, entity, tilemap);
    }
}

fn load_object<T: for<'a> Deserialize<'a>>(path: &str, file_name: &str) -> Result<T, SpannedError> {
    from_bytes(read_to_string(format!("{}{}", path, file_name))?.as_bytes())
}

fn complete<T: Component>(commands: &mut Commands, entity: Entity, component: T) {
    commands.entity(entity).remove::<TilemapLoader>();
    commands.entity(entity).insert(component);
}
