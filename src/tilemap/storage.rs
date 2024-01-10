use std::fmt::Debug;

use bevy::{
    math::IVec2,
    reflect::Reflect,
    utils::{HashMap, HashSet},
};

use crate::{
    math::{aabb::Aabb2d, extension::DivToFloor},
    DEFAULT_CHUNK_SIZE,
};

#[derive(Debug, Clone, Reflect)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub struct ChunkedStorage<T: Debug + Clone + Reflect> {
    pub chunk_size: u32,
    pub chunks: HashMap<IVec2, Vec<Option<T>>>,
    pub reserved: HashMap<IVec2, Aabb2d>,
    pub calc_queue: HashSet<IVec2>,
}

impl<T: Debug + Clone + Reflect> Default for ChunkedStorage<T> {
    fn default() -> Self {
        Self {
            chunk_size: DEFAULT_CHUNK_SIZE,
            chunks: HashMap::new(),
            reserved: HashMap::new(),
            calc_queue: HashSet::new(),
        }
    }
}

impl<T: Debug + Clone + Reflect> ChunkedStorage<T> {
    pub fn new(chunk_size: u32) -> Self {
        Self {
            chunk_size,
            ..Default::default()
        }
    }

    pub fn from_mapper(mapper: HashMap<IVec2, T>, chunk_size: Option<u32>) -> Self {
        let mut storage = Self::new(chunk_size.unwrap_or(32));
        mapper.into_iter().for_each(|(index, elem)| {
            storage.set_elem(index, Some(elem));
        });
        storage
    }

    pub fn get_elem(&self, index: IVec2) -> Option<&T> {
        let idx = self.transform_index(index);
        self.chunks
            .get(&idx.0)
            .and_then(|c| c.get(idx.1).as_ref().cloned())
            .and_then(|t| t.as_ref())
    }

    pub fn get_elem_mut(&mut self, index: IVec2) -> Option<&mut T> {
        let idx = self.transform_index(index);
        if let Some(chunk) = self.chunks.get_mut(&idx.0) {
            chunk.get_mut(idx.1).map(|t| t.as_mut()).flatten()
        } else {
            None
        }
    }

    pub fn set_elem(&mut self, index: IVec2, elem: Option<T>) {
        let idx = self.transform_index(index);
        self.queue_aabb(index);
        self.chunks
            .entry(idx.0)
            .or_insert_with(|| vec![None; (self.chunk_size * self.chunk_size) as usize])[idx.1] =
            elem;
    }

    pub fn set_elem_precise(&mut self, chunk_index: IVec2, in_chunk_index: usize, elem: Option<T>) {
        self.calc_queue.insert(chunk_index);
        self.chunks
            .entry(chunk_index)
            .or_insert_with(|| vec![None; (self.chunk_size * self.chunk_size) as usize])
            [in_chunk_index] = elem;
    }

    #[inline]
    pub fn get_chunk(&self, index: IVec2) -> Option<&Vec<Option<T>>> {
        self.chunks.get(&index)
    }

    #[inline]
    pub fn get_chunk_mut(&mut self, index: IVec2) -> Option<&mut Vec<Option<T>>> {
        self.chunks.get_mut(&index)
    }

    #[inline]
    pub fn get_chunk_or_insert(&mut self, index: IVec2) -> &mut Vec<Option<T>> {
        self.chunks
            .entry(index)
            .or_insert(vec![None; (self.chunk_size * self.chunk_size) as usize])
    }

    #[inline]
    pub fn set_chunk(&mut self, index: IVec2, chunk: Vec<Option<T>>) {
        self.queue_aabb(index);
        self.chunks.insert(index, chunk);
    }

    pub fn transform_index(&self, index: IVec2) -> (IVec2, usize) {
        let isize = IVec2::splat(self.chunk_size as i32);
        let c = index.div_to_floor(isize);
        let idx = index - c * isize;
        (c, (idx.y * isize.x + idx.x) as usize)
    }

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

    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Option<T>> {
        self.chunks.values_mut().map(|c| c.iter_mut()).flatten()
    }

    #[inline]
    pub fn remove_chunk(&mut self, index: IVec2) -> Option<Vec<Option<T>>> {
        self.chunks.remove(&index)
    }

    /// Declare that a chunk is existent.
    /// Use `reserve_with_aabb` if you can provide the aabb.
    /// Or `reserve_many` to reserve multiple chunks.
    #[inline]
    pub fn reserve(&mut self, index: IVec2) {
        self.calc_queue.insert(index);
    }

    #[inline]
    pub fn reserve_with_aabb(&mut self, index: IVec2, aabb: Aabb2d) {
        self.reserved.insert(index, aabb);
    }

    #[inline]
    pub fn reserve_many(&mut self, indices: impl Iterator<Item = IVec2>) {
        self.calc_queue.extend(indices);
    }

    #[inline]
    pub fn reserve_many_with_aabbs(&mut self, indices: impl Iterator<Item = (IVec2, Aabb2d)>) {
        self.reserved.extend(indices);
    }

    #[inline]
    fn queue_aabb(&mut self, index: IVec2) {
        if !self.reserved.contains_key(&index) {
            self.calc_queue.insert(index);
        }
    }
}
