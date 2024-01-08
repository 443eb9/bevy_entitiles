use std::{fmt::Debug, io::Write, path::Path};

use bevy::{
    ecs::{entity::Entity, system::Resource},
    math::IVec2,
    reflect::Reflect,
    utils::EntityHashMap,
};
use serde::Serialize;

#[cfg(feature = "algorithm")]
use crate::tilemap::algorithm::path::PathTile;
use crate::tilemap::{storage::ChunkedStorage, tile::TileBuilder};
pub mod load;
pub mod save;

pub trait ChunkCache<T: Debug + Clone + Reflect + Serialize> {
    fn get_cache(&self, entity: Entity) -> Option<&ChunkedStorage<T>>;
    fn get_cache_or_insert(&mut self, entity: Entity, chunk_size: u32) -> &mut ChunkedStorage<T>;
    fn write(&mut self, entity: Entity, chunk_index: IVec2, path: &Path, file_name: String) {
        self.get_cache(entity).map(|chunks| {
            chunks.get_chunk(chunk_index).map(|chunk| {
                std::fs::create_dir_all(path).unwrap_or_else(|err| panic!("{:?}", err));
                std::fs::File::create(path.join(file_name))
                    .map(|mut file| {
                        file.write(ron::to_string(&chunk).unwrap().as_bytes())
                            .unwrap_or_else(|err| panic!("{:?}", err));
                    })
                    .unwrap_or_else(|err| panic!("{:?}", err));
            });
        });
    }
}

#[derive(Resource, Default, Reflect)]
pub struct TilemapChunkCache {
    #[reflect(ignore)]
    pub chunks: EntityHashMap<Entity, ChunkedStorage<TileBuilder>>,
}

impl ChunkCache<TileBuilder> for TilemapChunkCache {
    #[inline]
    fn get_cache(&self, entity: Entity) -> Option<&ChunkedStorage<TileBuilder>> {
        self.chunks.get(&entity)
    }

    #[inline]
    fn get_cache_or_insert(
        &mut self,
        entity: Entity,
        chunk_size: u32,
    ) -> &mut ChunkedStorage<TileBuilder> {
        self.chunks
            .entry(entity)
            .or_insert(ChunkedStorage::new(chunk_size))
    }
}

#[cfg(feature = "algorithm")]
#[derive(Resource, Default, Reflect)]
pub struct TilemapPathChunkCache {
    #[reflect(ignore)]
    pub chunks: EntityHashMap<Entity, ChunkedStorage<PathTile>>,
}

#[cfg(feature = "algorithm")]
impl ChunkCache<PathTile> for TilemapPathChunkCache {
    #[inline]
    fn get_cache(&self, entity: Entity) -> Option<&ChunkedStorage<PathTile>> {
        self.chunks.get(&entity)
    }

    #[inline]
    fn get_cache_or_insert(
        &mut self,
        entity: Entity,
        chunk_size: u32,
    ) -> &mut ChunkedStorage<PathTile> {
        self.chunks
            .entry(entity)
            .or_insert(ChunkedStorage::new(chunk_size))
    }
}
