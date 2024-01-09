use std::path::Path;

use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        event::EventWriter,
        system::{Commands, ParallelCommands, Query},
    },
    math::IVec2,
    reflect::Reflect,
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

#[derive(Component, Debug, Clone, Reflect)]
pub struct TilemapChunkSaver {
    pub(crate) path: String,
    pub(crate) chunks: Vec<IVec2>,
    pub(crate) progress: usize,
    pub(crate) cpf: usize,
    pub(crate) remove_after_save: bool,
    pub(crate) layers: u32,
}

impl TilemapChunkSaver {
    /// For example if path = C:\\maps, then the crate will create:
    /// ```
    /// C
    /// └── maps
    ///     └── (your tilemap's name)
    ///         ├── tile_chunks
    ///         │   ├── 0_0.ron
    ///         │   ├── 0_1.ron
    ///         │   ...
    ///         ├── path_tile_chunks
    ///         ...
    /// ```
    pub fn new(path: String) -> Self {
        Self {
            path,
            chunks: vec![],
            remove_after_save: false,
            progress: 0,
            cpf: 1,
            layers: 0,
        }
    }

    pub fn with_single(mut self, chunk_index: IVec2) -> Self {
        self.chunks.push(chunk_index);
        self
    }

    pub fn with_range(mut self, start_index: IVec2, end_index: IVec2) -> Self {
        assert!(
            start_index.x <= end_index.x && start_index.y <= end_index.y,
            "start_index({}) must be less than (or equal to) end_index({})!",
            start_index,
            end_index
        );

        self.chunks
            .extend((start_index.y..=end_index.y).into_iter().flat_map(|y| {
                (start_index.x..=end_index.x)
                    .into_iter()
                    .map(move |x| IVec2 { x, y })
            }));
        self
    }

    pub fn with_multiple_ranges(mut self, ranges: Vec<IAabb2d>) -> Self {
        self.chunks
            .extend(ranges.iter().flat_map(|aabb| (*aabb).into_iter()));
        self
    }

    pub fn remove_after_save(mut self) -> Self {
        self.remove_after_save = true;
        self
    }

    pub fn with_chunks_per_frame(mut self, chunks_per_frame: usize) -> Self {
        self.cpf = chunks_per_frame;
        self
    }

    pub fn with_layer(mut self, layer: TilemapLayer) -> Self {
        self.layers |= layer as u32;
        self
    }
}

#[derive(Component, Debug, Clone, Reflect)]
pub struct TilemapPathChunkSaver {
    pub(crate) path: String,
    pub(crate) chunks: Vec<IVec2>,
    pub(crate) progress: usize,
    pub(crate) cpf: usize,
    pub(crate) remove_after_save: bool,
}

/// As this operation is performance heavy, the crate will do it asynchronously by default.
/// But the target chunk(s) will be excluded from rendering immediately.
#[derive(Component, Debug, Clone, Reflect)]
pub struct TilemapColorChunkSaver {
    pub(crate) path: String,
    pub(crate) chunks: Vec<IVec2>,
    pub(crate) progress: usize,
    pub(crate) cpf: usize,
    pub(crate) remove_after_save: bool,
}

pub fn saver_expander(
    commands: ParallelCommands,
    tilemaps_query: Query<(Entity, &TilemapChunkSaver)>,
) {
    tilemaps_query.par_iter().for_each(|(entity, saver)| {
        commands.command_scope(|mut c| {
            if (saver.layers & TilemapLayer::Color as u32) != 0 {
                c.entity(entity).insert(TilemapColorChunkSaver {
                    path: saver.path.clone(),
                    chunks: saver.chunks.clone(),
                    progress: 0,
                    cpf: saver.cpf,
                    remove_after_save: saver.remove_after_save,
                });
            }

            if (saver.layers & TilemapLayer::Path as u32) != 0 {
                c.entity(entity).insert(TilemapPathChunkSaver {
                    path: saver.path.clone(),
                    chunks: saver.chunks.clone(),
                    progress: 0,
                    cpf: saver.cpf,
                    remove_after_save: saver.remove_after_save,
                });
            }

            c.entity(entity).remove::<TilemapChunkSaver>();
        });
    });
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
    mut tilemaps_query: Query<(
        Entity,
        &TilemapName,
        &mut TilemapStorage,
        &mut TilemapColorChunkSaver,
    )>,
    tiles_query: Query<&Tile>,
    mut chunk_unload: EventWriter<ChunkUnload>,
) {
    tilemaps_query.for_each_mut(|(entity, name, mut storage, mut saver)| {
        let map_path = Path::new(&saver.path).join(&name.0);
        let tiles_per_frame = saver.cpf;
        let tile_progress = saver.progress;
        let chunk_area = (storage.storage.chunk_size * storage.storage.chunk_size) as usize;
        let remove_after_save = saver.remove_after_save;

        (tile_progress..tile_progress + tiles_per_frame)
            .into_iter()
            .filter_map(|i| saver.chunks.get(i).cloned())
            .for_each(|chunk_index| {
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

        if saver.progress / chunk_area >= saver.chunks.len() {
            commands.entity(entity).remove::<TilemapColorChunkSaver>();
        }
        saver.progress += tiles_per_frame;
    });
}

#[cfg(feature = "algorithm")]
pub fn save_path_layer(
    mut commands: Commands,
    mut tilemaps_query: Query<(
        Entity,
        &TilemapName,
        &mut TilemapPathChunkSaver,
        &mut PathTilemap,
    )>,
) {
    use super::PATH_TILE_CHUNKS_FOLDER;

    tilemaps_query.for_each_mut(|(entity, name, mut saver, mut path_tilemap)| {
        let map_path = Path::new(&saver.path).join(&name.0);
        let chunks_per_frame = saver.cpf;
        let chunk_progress = saver.progress;
        let remove_after_save = saver.remove_after_save;

        (chunk_progress..chunk_progress + chunks_per_frame)
            .into_iter()
            .filter_map(|i| saver.chunks.get(i).cloned())
            .for_each(|chunk_index| {
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

        if saver.cpf >= saver.chunks.len() {
            commands.entity(entity).remove::<TilemapPathChunkSaver>();
        }
        saver.cpf += chunks_per_frame;
    });
}
