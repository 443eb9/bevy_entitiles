use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        system::{ParallelCommands, Query},
    },
    math::IVec2,
    reflect::Reflect,
};

use crate::{
    math::{aabb::IAabb2d, extension::ChunkIndex},
    render::chunk::UnloadedRenderChunk,
    serializing::map::{SerializedTile, TilemapLayer},
    tilemap::{
        map::{TilemapName, TilemapStorage},
        tile::Tile,
    },
};

pub const TILE_CHUNKS_FOLDER: &str = "tile_chunks";
pub const PATH_TILE_CHUNKS_FOLDER: &str = "path_tile_chunks";

#[derive(Component, Debug, Clone, Reflect)]
pub struct TilemapChunkSaver {
    pub(crate) path: String,
    pub(crate) layer: u32,
    pub(crate) ranges: Vec<IAabb2d>,
    pub(crate) remove_after_save: bool,
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
    ///
    /// If the mode is `TilemapSaverMode::MapPattern`, then the crate will create:
    /// ```
    /// C
    /// └── maps
    ///     └── (your tilemap's name).pattern
    /// ```
    pub fn new(path: String) -> Self {
        Self {
            path,
            layer: 0,
            ranges: vec![],
            remove_after_save: false,
        }
    }

    pub fn with_layer(mut self, layer: TilemapLayer) -> Self {
        self.layer |= layer as u32;
        self
    }

    pub fn with_single(mut self, chunk_index: IVec2) -> Self {
        self.ranges.push(IAabb2d {
            min: chunk_index,
            max: chunk_index,
        });
        self
    }

    pub fn with_range(mut self, start_index: IVec2, end_index: IVec2) -> Self {
        self.ranges.push(IAabb2d {
            min: start_index,
            max: end_index,
        });
        self
    }

    pub fn with_multiple_ranges(mut self, ranges: Vec<IAabb2d>) -> Self {
        self.ranges = ranges;
        self
    }

    pub fn remove_after_save(mut self) -> Self {
        self.remove_after_save = true;
        self
    }
}

pub fn save(
    commands: ParallelCommands,
    mut tilemaps_query: Query<(
        Entity,
        &TilemapName,
        &mut TilemapStorage,
        &TilemapChunkSaver,
        Option<&UnloadedRenderChunk>,
    )>,
    tiles_query: Query<&Tile>,
    #[cfg(feature = "algorithm")] mut path_tilemaps_query: Query<
        &mut crate::tilemap::algorithm::path::PathTilemap,
    >,
) {
    tilemaps_query
        .iter_mut()
        .for_each(|(entity, name, mut storage, saver, unloaded)| {
            let chunks = saver
                .ranges
                .iter()
                .flat_map(|aabb| aabb.into_iter())
                .collect::<Vec<_>>();
            let map_path = Path::new(&saver.path).join(&name.0);

            if saver.layer & TilemapLayer::Color as u32 != 0 {
                chunks
                    .iter()
                    .flat_map(|chunk_index| {
                        storage
                            .storage
                            .chunks
                            .get(chunk_index)
                            .map(|c| (chunk_index, c))
                    })
                    .map(|(chunk_index, chunk)| {
                        (
                            chunk_index,
                            chunk
                                .iter()
                                .filter_map(|t_e| {
                                    t_e.map(|e| {
                                        tiles_query.get(e).ok().map(|t| {
                                            <Tile as Into<SerializedTile>>::into(t.clone())
                                        })
                                    })
                                })
                                .filter_map(|t| t)
                                .collect::<Vec<_>>(),
                        )
                    })
                    .for_each(|(chunk_index, chunk)| {
                        save_object(
                            &map_path,
                            TILE_CHUNKS_FOLDER,
                            format!("{}.ron", chunk_index.chunk_file_name()).as_str(),
                            &chunk,
                        );
                    });

                if saver.remove_after_save {
                    commands.command_scope(|mut c| {
                        chunks.iter().for_each(|ci| {
                            storage.remove_chunk(&mut c, *ci);
                        });
                        let mut unloaded =
                            unloaded.as_ref().map(|u| u.0.clone()).unwrap_or_default();
                        unloaded.extend(&chunks);
                        c.entity(entity).insert(UnloadedRenderChunk(unloaded));
                    });
                }
            }

            #[cfg(feature = "algorithm")]
            if saver.layer & TilemapLayer::Algorithm as u32 != 0 {
                if let Ok(mut path_tilemap) = path_tilemaps_query.get_mut(entity) {
                    chunks
                        .iter()
                        .filter_map(|chunk_index| {
                            path_tilemap
                                .storage
                                .chunks
                                .get(chunk_index)
                                .map(|c| (chunk_index, c))
                        })
                        .map(|(ci, c)| (ci, c.iter().filter_map(|t| t.clone()).collect::<Vec<_>>()))
                        .for_each(|(chunk_index, chunk)| {
                            save_object(
                                &map_path,
                                PATH_TILE_CHUNKS_FOLDER,
                                format!("{}.ron", chunk_index.chunk_file_name()).as_str(),
                                &chunk,
                            );
                        });

                    if saver.remove_after_save {
                        chunks.iter().for_each(|ci| {
                            path_tilemap.storage.remove_chunk(*ci);
                        });
                    }
                }
            }

            commands.command_scope(|mut c| {
                c.entity(entity).remove::<TilemapChunkSaver>();
            });
        });
}

fn save_object(base_path: &Path, folder: &str, file_name: &str, object: &impl serde::Serialize) {
    let path = base_path.join(folder);
    fs::create_dir_all(&path).unwrap_or_else(|err| panic!("{:?}", err));
    let path = path.join(file_name);
    File::create(path.clone())
        .unwrap_or(File::open(path).unwrap())
        .write(ron::to_string(object).unwrap().as_bytes())
        .unwrap_or_else(|err| panic!("{:?}", err));
}
