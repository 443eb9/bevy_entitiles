use std::fmt::Debug;

use bevy::{math::IVec2, reflect::Reflect, utils::HashMap};

use crate::{math::extension::DivToFloor, DEFAULT_CHUNK_SIZE};

#[derive(Debug, Clone, Reflect)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub struct ChunkedStorage<T: Debug + Clone + Reflect> {
    pub chunk_size: u32,
    pub chunks: HashMap<IVec2, Vec<Option<T>>>,
}

impl<T: Debug + Clone + Reflect> Default for ChunkedStorage<T> {
    fn default() -> Self {
        Self {
            chunk_size: DEFAULT_CHUNK_SIZE,
            chunks: Default::default(),
        }
    }
}

impl<T: Debug + Clone + Reflect> ChunkedStorage<T> {
    pub fn new(chunk_size: u32) -> Self {
        Self {
            chunk_size,
            chunks: HashMap::new(),
        }
    }

    pub fn from_mapper(mapper: HashMap<IVec2, T>, chunk_size: Option<u32>) -> Self {
        let mut storage = Self::new(chunk_size.unwrap_or(32));
        mapper.into_iter().for_each(|(index, elem)| {
            storage.set(index, Some(elem));
        });
        storage
    }

    pub fn get(&self, index: IVec2) -> Option<&T> {
        let idx = self.transform_index(index);
        self.chunks
            .get(&idx.0)
            .and_then(|c| c.get(idx.1).as_ref().cloned())
            .and_then(|t| t.as_ref())
    }

    pub fn get_mut(&mut self, index: IVec2) -> Option<&mut T> {
        let idx = self.transform_index(index);
        if let Some(chunk) = self.chunks.get_mut(&idx.0) {
            chunk.get_mut(idx.1).map(|t| t.as_mut()).flatten()
        } else {
            None
        }
    }

    pub fn set(&mut self, index: IVec2, elem: Option<T>) {
        let idx = self.transform_index(index);
        self.chunks
            .entry(idx.0)
            .or_insert_with(|| vec![None; (self.chunk_size * self.chunk_size) as usize])[idx.1] =
            elem;
    }

    pub fn transform_index(&self, index: IVec2) -> (IVec2, usize) {
        let isize = IVec2::splat(self.chunk_size as i32);
        let c = index.div_to_floor(isize);
        let idx = index - c * isize;
        (c, (idx.y * isize.x + idx.x) as usize)
    }

    #[inline]
    pub fn into_mapper(mut self) -> HashMap<IVec2, T> {
        let mut mapper = HashMap::new();
        self.chunks.drain().for_each(|(chunk_index, chunk)| {
            chunk.into_iter().enumerate().for_each(|(index, elem)| {
                if let Some(elem) = elem {
                    mapper.insert(
                        chunk_index * IVec2::splat(self.chunk_size as i32)
                            + IVec2 {
                                x: index as i32 % self.chunk_size as i32,
                                y: index as i32 / self.chunk_size as i32,
                            },
                        elem,
                    );
                }
            });
        });
        mapper
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &Option<T>> {
        self.chunks.values().map(|c| c.iter()).flatten()
    }
}
