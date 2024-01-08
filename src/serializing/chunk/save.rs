use std::path::Path;

use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        system::{Commands, Query},
    },
    math::IVec2,
    reflect::Reflect,
};

use crate::{
    impl_chunk_saver,
    math::{aabb::IAabb2d, extension::ChunkIndex},
    render::chunk::UnloadRenderChunk,
    serializing::save_object,
    tilemap::{
        map::{TilemapName, TilemapStorage},
        tile::{Tile, TileBuilder},
    },
};

#[cfg(feature = "algorithm")]
use crate::tilemap::algorithm::path::PathTilemap;

pub const TILE_CHUNKS_FOLDER: &str = "tile_chunks";
pub const PATH_TILE_CHUNKS_FOLDER: &str = "path_tile_chunks";

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
pub struct TilemapChunkSaver {
    pub(crate) path: String,
    pub(crate) chunks: Vec<IVec2>,
    pub(crate) progress: usize,
    pub(crate) cpf: usize,
    pub(crate) remove_after_save: bool,
}

impl_chunk_saver!(TilemapChunkSaver);
impl_chunk_saver!(TilemapPathChunkSaver);

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
        &mut TilemapChunkSaver,
        Option<&UnloadRenderChunk>,
    )>,
    tiles_query: Query<&Tile>,
) {
    tilemaps_query.for_each_mut(|(entity, name, mut storage, mut saver, unloaded)| {
        let map_path = Path::new(&saver.path).join(&name.0);
        let tiles_per_frame = saver.cpf;
        let tile_progress = saver.progress;
        let chunk_area = (storage.storage.chunk_size * storage.storage.chunk_size) as usize;
        let remove_after_save = saver.remove_after_save;

        let mut unloaded = unloaded.as_ref().map(|u| u.0.clone()).unwrap_or_default();

        (tile_progress..tile_progress + tiles_per_frame)
            .into_iter()
            .filter_map(|i| saver.chunks.get(i).cloned())
            .for_each(|chunk_index| {
                if let Some(chunk) = storage.get_chunk(chunk_index) {
                    save_object(
                        &map_path.join(TILE_CHUNKS_FOLDER),
                        format!("{}.ron", chunk_index.chunk_file_name()).as_str(),
                        &chunk
                            .iter()
                            .map(|t| {
                                t.and_then(|e| tiles_query.get(e).ok().cloned())
                                    .map(|tile| <Tile as Into<TileBuilder>>::into(tile))
                            })
                            .collect::<Vec<_>>(),
                    );

                    if remove_after_save {
                        storage.remove_chunk(&mut commands, chunk_index);
                        unloaded.push(chunk_index);
                    }
                }
            });

        if saver.progress / chunk_area >= saver.chunks.len() {
            commands.entity(entity).remove::<TilemapChunkSaver>();
        }
        saver.progress += tiles_per_frame;

        if remove_after_save {
            commands.entity(entity).insert(UnloadRenderChunk(unloaded));
        }
    });
}

#[cfg(feature = "algorithm")]
pub fn save_algo_layer(
    mut commands: Commands,
    mut tilemaps_query: Query<(
        Entity,
        &TilemapName,
        &mut TilemapPathChunkSaver,
        &mut PathTilemap,
    )>,
) {
    tilemaps_query.for_each_mut(|(entity, name, mut saver, mut path_tilemap)| {
        let map_path = Path::new(&saver.path).join(&name.0);
        let chunks_per_frame = saver.cpf;
        let chunk_progress = saver.progress;
        let remove_after_save = saver.remove_after_save;

        (chunk_progress..chunk_progress + chunks_per_frame)
            .into_iter()
            .filter_map(|i| saver.chunks.get(i).cloned())
            .for_each(|chunk_index| {
                if let Some(chunk) = path_tilemap.storage.get_chunk(chunk_index) {
                    save_object(
                        &map_path.join(PATH_TILE_CHUNKS_FOLDER),
                        format!("{}.ron", chunk_index.chunk_file_name()).as_str(),
                        chunk,
                    );
                }

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
