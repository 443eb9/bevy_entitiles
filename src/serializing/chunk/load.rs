use std::path::Path;

use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        system::{ParallelCommands, Query},
    },
    hierarchy::BuildChildren,
    math::IVec2,
    reflect::Reflect,
};

use crate::{
    math::{aabb::IAabb2d, extension::ChunkIndex},
    serializing::{load_object, map::TilemapLayer},
    tilemap::{buffers::TileBuilderBuffer, map::TilemapStorage, tile::Tile},
};

#[cfg(feature = "algorithm")]
use super::PATH_TILE_CHUNKS_FOLDER;
#[cfg(feature = "algorithm")]
use crate::tilemap::{algorithm::path::PathTilemap, buffers::PathTilesBuffer};

use super::TILE_CHUNKS_FOLDER;

#[derive(Component, Debug, Clone, Reflect)]
pub struct TilemapChunkLoader {
    pub(crate) path: String,
    pub(crate) map_name: String,
    pub(crate) chunks: Vec<IVec2>,
    pub(crate) progress: usize,
    pub(crate) cpf: usize,
    pub(crate) layers: u32,
}

impl TilemapChunkLoader {
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
        Self {
            path,
            map_name,
            chunks: Vec::new(),
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
pub struct TilemapColorChunkLoader {
    pub(crate) path: String,
    pub(crate) map_name: String,
    pub(crate) chunks: Vec<IVec2>,
    pub(crate) progress: usize,
    pub(crate) cpf: usize,
}

#[derive(Component, Debug, Clone, Reflect)]
pub struct TilemapPathChunkLoader {
    pub(crate) path: String,
    pub(crate) map_name: String,
    pub(crate) chunks: Vec<IVec2>,
    pub(crate) progress: usize,
    pub(crate) cpf: usize,
}

pub fn loader_expander(
    commands: ParallelCommands,
    tilemaps_query: Query<(Entity, &TilemapChunkLoader)>,
) {
    tilemaps_query.par_iter().for_each(|(entity, saver)| {
        commands.command_scope(|mut c| {
            if (saver.layers & TilemapLayer::Color as u32) != 0 {
                c.entity(entity).insert(TilemapColorChunkLoader {
                    path: saver.path.clone(),
                    map_name: saver.map_name.clone(),
                    chunks: saver.chunks.clone(),
                    progress: 0,
                    cpf: saver.cpf,
                });
            }

            if (saver.layers & TilemapLayer::Path as u32) != 0 {
                c.entity(entity).insert(TilemapColorChunkLoader {
                    path: saver.path.clone(),
                    map_name: saver.map_name.clone(),
                    chunks: saver.chunks.clone(),
                    progress: 0,
                    cpf: saver.cpf,
                });
            }

            c.entity(entity).remove::<TilemapChunkLoader>();
        });
    });
}

pub fn load_color_layer(
    commands: ParallelCommands,
    mut tilemaps_query: Query<(Entity, &mut TilemapStorage, &mut TilemapColorChunkLoader)>,
) {
    tilemaps_query
        .par_iter_mut()
        .for_each(|(entity, mut storage, mut loader)| {
            let chunk_size = storage.storage.chunk_size as i32;
            (loader.progress..loader.progress + loader.cpf)
                .into_iter()
                .for_each(|i| {
                    let chunk_index = loader.chunks[i];
                    let Ok(chunk) = load_object::<TileBuilderBuffer>(
                        &Path::new(&loader.path)
                            .join(loader.map_name.clone())
                            .join(TILE_CHUNKS_FOLDER),
                        format!("{}.ron", chunk_index.chunk_file_name()).as_str(),
                    ) else {
                        return;
                    };

                    commands.command_scope(|mut c| {
                        let mut tiles = Vec::with_capacity((chunk_size * chunk_size) as usize);
                        let mut entities = vec![None; (chunk_size * chunk_size) as usize];
                        let chunk_origin = chunk_index * chunk_size;
                        chunk.tiles.into_iter().for_each(|(in_chunk_index, tile)| {
                            let e = c.spawn_empty().id();
                            tiles.push((
                                e,
                                Tile {
                                    tilemap_id: entity,
                                    chunk_index,
                                    in_chunk_index: in_chunk_index.as_uvec2(),
                                    index: chunk_origin + in_chunk_index,
                                    texture: tile.texture,
                                    color: tile.color,
                                },
                            ));
                            entities[(in_chunk_index.y * chunk_size + in_chunk_index.x) as usize] =
                                Some(e);
                        });

                        c.entity(entity)
                            .push_children(&entities.iter().filter_map(|e| *e).collect::<Vec<_>>());
                        storage.storage.set_chunk(chunk_index, entities);
                        c.insert_or_spawn_batch(tiles);
                    });
                });

            loader.progress += loader.cpf;
            if loader.progress >= loader.chunks.len() {
                commands.command_scope(|mut c| {
                    c.entity(entity).remove::<TilemapColorChunkLoader>();
                });
            }
        });
}

#[cfg(feature = "algorithm")]
pub fn load_path_layer(
    commands: ParallelCommands,
    mut tilemaps_query: Query<(Entity, &mut PathTilemap, &mut TilemapPathChunkLoader)>,
) {
    tilemaps_query
        .par_iter_mut()
        .for_each(|(entity, path_tilemap, mut loader)| {
            let chunk_size = path_tilemap.storage.chunk_size as i32;
            (loader.progress..loader.progress + loader.cpf)
                .into_iter()
                .for_each(|i| {
                    let chunk_index = loader.chunks[i];
                    let Ok(chunk) = load_object::<PathTilesBuffer>(
                        &Path::new(&loader.path)
                            .join(loader.map_name.clone())
                            .join(PATH_TILE_CHUNKS_FOLDER),
                        format!("{}.ron", chunk_index.chunk_file_name()).as_str(),
                    ) else {
                        return;
                    };

                    let mut c = vec![None; (chunk_size * chunk_size) as usize];
                    chunk.tiles.into_iter().for_each(|(in_chunk_index, tile)| {
                        c[(in_chunk_index.y * chunk_size + in_chunk_index.x) as usize] = Some(tile);
                    });
                    path_tilemap.storage.get_chunk(chunk_index).replace(&c);
                });

            loader.progress += loader.cpf;
            if loader.progress >= loader.chunks.len() {
                commands.command_scope(|mut c| {
                    c.entity(entity).remove::<TilemapPathChunkLoader>();
                });
            }
        });
}
