use std::{collections::VecDeque, path::Path};

use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        event::EventWriter,
        query::With,
        system::{Commands, Query, Res, ResMut, Resource},
    },
    math::IVec2,
    reflect::Reflect,
    utils::{EntityHashMap, HashMap},
};

use crate::{
    math::{aabb::IAabb2d, extension::ChunkIndex},
    render::chunk::{ChunkUnload, UnloadRenderChunk},
    serializing::{map::TilemapLayer, save_object},
    tilemap::{
        buffers::TileBuilderBuffer,
        map::{TilemapName, TilemapStorage},
        tile::Tile,
    },
};

#[cfg(feature = "algorithm")]
use crate::tilemap::{algorithm::path::PathTilemap, buffers::PathTilesBuffer};

use super::TILE_CHUNKS_FOLDER;

#[derive(Component)]
pub struct ScheduledSaveChunks;

#[derive(Resource, Default, Reflect)]
pub struct ChunkSaveConfig {
    pub path: String,
    pub chunks_per_frame: usize,
}

#[derive(Resource, Default)]
pub struct ChunkSaveCache(
    pub(crate) EntityHashMap<Entity, HashMap<TilemapLayer, VecDeque<(IVec2, bool)>>>,
);

impl ChunkSaveCache {
    #[inline]
    pub fn schedule(
        &mut self,
        commands: &mut Commands,
        tilemap: Entity,
        layer: TilemapLayer,
        chunk_index: IVec2,
        remove_after_save: bool,
    ) {
        self.0
            .entry(tilemap)
            .or_default()
            .entry(layer)
            .or_default()
            .push_front((chunk_index, remove_after_save));
        commands.entity(tilemap).insert(ScheduledSaveChunks);
    }

    #[inline]
    pub fn schedule_many(
        &mut self,
        commands: &mut Commands,
        tilemap: Entity,
        layer: TilemapLayer,
        chunk_indices: impl Iterator<Item = (IVec2, bool)>,
    ) {
        let queue = self.0.entry(tilemap).or_default().entry(layer).or_default();
        queue.reserve(chunk_indices.size_hint().0);
        chunk_indices.for_each(|chunk_index| queue.push_front(chunk_index));
        commands.entity(tilemap).insert(ScheduledSaveChunks);
    }

    #[inline]
    pub fn pop_chunk(&mut self, tilemap: Entity, layer: TilemapLayer) -> Option<(IVec2, bool)> {
        self.0
            .get_mut(&tilemap)
            .map(|layers| {
                layers
                    .get_mut(&layer)
                    .map(|chunks| chunks.pop_back())
                    .flatten()
            })
            .flatten()
    }
}

pub fn render_chunk_remover(mut tilemaps_query: Query<(&mut TilemapStorage, &UnloadRenderChunk)>) {
    tilemaps_query
        .par_iter_mut()
        .for_each(|(mut storage, unloaded)| {
            unloaded.0.iter().for_each(|chunk_index| {
                storage.storage.chunks.remove(chunk_index);
            });
        });
}

pub fn save_color_layer(
    mut commands: Commands,
    mut tilemaps_query: Query<
        (Entity, &TilemapName, &mut TilemapStorage),
        With<ScheduledSaveChunks>,
    >,
    tiles_query: Query<&Tile>,
    mut chunk_unload: EventWriter<ChunkUnload>,
    config: Res<ChunkSaveConfig>,
    mut cache: ResMut<ChunkSaveCache>,
) {
    tilemaps_query.for_each_mut(|(entity, name, mut storage)| {
        let map_path = Path::new(&config.path).join(&name.0);

        (0..config.chunks_per_frame).into_iter().for_each(|_| {
            let Some((chunk_index, remove_after_save)) =
                cache.pop_chunk(entity, TilemapLayer::Color)
            else {
                cache
                    .0
                    .get_mut(&entity)
                    .unwrap()
                    .remove(&TilemapLayer::Color);
                return;
            };

            let Some(chunk) = storage.get_chunk(chunk_index) else {
                return;
            };

            let tiles = chunk
                .iter()
                .enumerate()
                .filter_map(|(index, t)| {
                    t.map(|t| {
                        (
                            IVec2 {
                                x: (index as u32 % storage.storage.chunk_size) as i32,
                                y: (index as u32 / storage.storage.chunk_size) as i32,
                            },
                            tiles_query
                                .get(t)
                                .ok()
                                .cloned()
                                .map(|tile| tile.into())
                                .unwrap(),
                        )
                    })
                })
                .collect();

            save_object(
                &map_path.join(TILE_CHUNKS_FOLDER),
                format!("{}.ron", chunk_index.chunk_file_name()).as_str(),
                &TileBuilderBuffer {
                    tiles,
                    aabb: IAabb2d {
                        min: IVec2::ZERO,
                        max: IVec2::splat(storage.storage.chunk_size as i32 - 1),
                    },
                },
            );

            if remove_after_save {
                storage.remove_chunk(&mut commands, chunk_index);
                chunk_unload.send(ChunkUnload {
                    tilemap: entity,
                    index: chunk_index,
                });
            }
        });
    });
}

#[cfg(feature = "algorithm")]
pub fn save_path_layer(
    mut tilemaps_query: Query<(Entity, &TilemapName, &mut PathTilemap)>,
    config: Res<ChunkSaveConfig>,
    mut cache: ResMut<ChunkSaveCache>,
) {
    use super::PATH_TILE_CHUNKS_FOLDER;

    tilemaps_query.for_each_mut(|(entity, name, mut path_tilemap)| {
        let map_path = Path::new(&config.path).join(&name.0);

        (0..config.chunks_per_frame).into_iter().for_each(|_| {
            let Some((chunk_index, remove_after_save)) =
                cache.pop_chunk(entity, TilemapLayer::Path)
            else {
                cache
                    .0
                    .get_mut(&entity)
                    .unwrap()
                    .remove(&TilemapLayer::Path);
                return;
            };

            let Some(chunk) = path_tilemap.storage.get_chunk(chunk_index) else {
                return;
            };

            let tiles = chunk
                .iter()
                .enumerate()
                .filter_map(|(index, tile)| {
                    tile.map(|t| {
                        (
                            IVec2 {
                                x: (index as u32 % path_tilemap.storage.chunk_size) as i32,
                                y: (index as u32 / path_tilemap.storage.chunk_size) as i32,
                            },
                            t,
                        )
                    })
                })
                .collect();

            save_object(
                &map_path.join(PATH_TILE_CHUNKS_FOLDER),
                format!("{}.ron", chunk_index.chunk_file_name()).as_str(),
                &PathTilesBuffer {
                    tiles,
                    aabb: IAabb2d {
                        min: IVec2::ZERO,
                        max: IVec2::splat(path_tilemap.storage.chunk_size as i32 - 1),
                    },
                },
            );

            if remove_after_save {
                path_tilemap.storage.chunks.remove(&chunk_index);
            }
        });
    });
}
