use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        system::{Commands, Query, ResMut},
    },
    math::IVec2,
    reflect::Reflect,
    utils::HashMap, hierarchy::DespawnRecursiveExt,
};

use crate::{
    math::{aabb::IAabb2d, extension::ChunkIndex},
    render::chunk::UnloadRenderChunk,
    serializing::map::TilemapLayer,
    tilemap::{
        map::{TilemapName, TilemapStorage},
        storage::ChunkedStorage,
        tile::{Tile, TileBuilder},
    },
};

use super::{ChunkCache, TilemapChunkCache};

pub const TILE_CHUNKS_FOLDER: &str = "tile_chunks";
pub const PATH_TILE_CHUNKS_FOLDER: &str = "path_tile_chunks";

/// As this operation is performance heavy, the crate will do it asynchronously by default.
/// But the target chunk(s) will be excluded from rendering immediately.
#[derive(Component, Debug, Clone, Reflect)]
pub struct TilemapChunkUnloader {
    pub(crate) path: String,
    pub(crate) layer: u32,
    /// The usize is the progress
    pub(crate) chunks: HashMap<TilemapLayer, (Vec<IVec2>, usize)>,
    pub(crate) entity_cache: Option<ChunkedStorage<Entity>>,
    pub(crate) tiles_per_frame: usize,
    pub(crate) remove_after_save: bool,
}

impl TilemapChunkUnloader {
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
            chunks: HashMap::new(),
            remove_after_save: false,
            tiles_per_frame: 32,
            entity_cache: None,
        }
    }

    pub fn with_layer(mut self, layer: TilemapLayer) -> Self {
        self.layer |= layer as u32;
        self.chunks.insert(layer, (Vec::new(), 0));
        self
    }

    pub fn with_single(mut self, chunk_index: IVec2) -> Self {
        self.chunks.values_mut().for_each(|v| v.0.push(chunk_index));
        self
    }

    pub fn with_range(mut self, start_index: IVec2, end_index: IVec2) -> Self {
        assert!(
            start_index.x <= end_index.x && start_index.y <= end_index.y,
            "start_index({}) must be less than (or equal to) end_index({})!",
            start_index,
            end_index
        );

        self.chunks.values_mut().for_each(|v| {
            v.0.extend((start_index.y..=end_index.y).into_iter().flat_map(|y| {
                (start_index.x..=end_index.x)
                    .into_iter()
                    .map(move |x| IVec2 { x, y })
            }));
        });
        self
    }

    pub fn with_multiple_ranges(mut self, ranges: Vec<IAabb2d>) -> Self {
        self.chunks.values_mut().for_each(|v| {
            v.0.extend(ranges.iter().flat_map(|aabb| (*aabb).into_iter()));
        });
        self
    }

    pub fn remove_after_save(mut self) -> Self {
        self.remove_after_save = true;
        self
    }

    pub fn with_tiles_per_frame(mut self, tiles_per_frame: usize) -> Self {
        self.tiles_per_frame = tiles_per_frame;
        self
    }
}

pub fn save(
    mut commands: Commands,
    mut tilemaps_query: Query<(
        Entity,
        &TilemapName,
        &mut TilemapStorage,
        &mut TilemapChunkUnloader,
        Option<&UnloadRenderChunk>,
    )>,
    tiles_query: Query<&Tile>,
    #[cfg(feature = "algorithm")] mut path_tilemaps_query: Query<
        &mut crate::tilemap::algorithm::path::PathTilemap,
    >,
    mut tile_cache: ResMut<TilemapChunkCache>,
) {
    tilemaps_query
        .iter_mut()
        .for_each(|(entity, name, mut storage, mut saver, unloaded)| {
            let map_path = Path::new(&saver.path).join(&name.0);
            let tiles_per_frame = saver.tiles_per_frame;
            let chunk_area = (storage.storage.chunk_size * storage.storage.chunk_size) as usize;
            let remove_after_save = saver.remove_after_save;

            if saver.layer & TilemapLayer::Color as u32 != 0 {
                let cur_chunk = saver.chunks.get_mut(&TilemapLayer::Color).unwrap();
                (cur_chunk.1..cur_chunk.1 + tiles_per_frame)
                    .into_iter()
                    .filter_map(|i| {
                        if let Some(chunk_index) = cur_chunk.0.get(i / chunk_area) {
                            Some((*chunk_index, i % chunk_area))
                        } else {
                            None
                        }
                    })
                    .for_each(|(chunk_index, in_chunk_index)| {
                        if let Some(tile) = storage
                            .get_chunk(chunk_index)
                            .and_then(|c| c[in_chunk_index])
                        {
                            tile_cache
                                .get_cache_or_insert(entity, storage.storage.chunk_size)
                                .get_chunk_or_insert(chunk_index)[in_chunk_index] = tiles_query
                                .get(tile)
                                .ok()
                                .and_then(|t| Some(t.clone().into()));

                            if remove_after_save {
                                commands.entity(tile).despawn();
                            }
                        }

                        if in_chunk_index == chunk_area - 1 {
                            tile_cache.write(
                                entity,
                                chunk_index,
                                &map_path.join(TILE_CHUNKS_FOLDER),
                                format!("{}.ron", chunk_index.chunk_file_name()),
                            );

                            if remove_after_save {
                                let mut unloaded =
                                    unloaded.as_ref().map(|u| u.0.clone()).unwrap_or_default();
                                unloaded.push(chunk_index);
                                storage.storage.chunks.remove(&chunk_index);
                                commands.entity(entity).insert(UnloadRenderChunk(unloaded));
                            }
                        }
                    });

                cur_chunk.1 += tiles_per_frame;
                if cur_chunk.1 / chunk_area >= cur_chunk.0.len() {
                    saver.chunks.remove(&TilemapLayer::Color);
                }
            }

            // #[cfg(feature = "algorithm")]
            // if saver.layer & TilemapLayer::Algorithm as u32 != 0 {
            //     if let Ok(mut path_tilemap) = path_tilemaps_query.get_mut(entity) {
            //         let cur_chunk = saver.chunks.get_mut(&TilemapLayer::Algorithm).unwrap();
            //         let Some(target_chunk) = cur_chunk.0.last().cloned() else {
            //             saver.chunks.remove(&TilemapLayer::Algorithm);
            //             return;
            //         };

            //         path_tilemap.storage.get_chunk(target_chunk).map(|chunk| {
            //             save_object(
            //                 &map_path,
            //                 PATH_TILE_CHUNKS_FOLDER,
            //                 format!("{}.ron", target_chunk.chunk_file_name()).as_str(),
            //                 &chunk,
            //             );
            //         });

            //         if saver.remove_after_save {
            //             path_tilemap.storage.remove_chunk(target_chunk);
            //         }
            //     }
            // }

            if saver.chunks.is_empty() {
                commands.entity(entity).remove::<TilemapChunkUnloader>();
            }
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
