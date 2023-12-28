use std::fmt::Debug;

use bevy::{
    prelude::{Plugin, Update},
    reflect::Reflect,
    utils::HashMap,
};

use self::{
    pathfinding::{
        pathfinding, pathfinding_async, AsyncPathfinder, Path, PathGrid, PathNode, PathTile,
        Pathfinder,
    },
    wfc::{
        wave_function_collapse, wave_function_collapse_async, AsyncWfcRunner, WfcElement,
        WfcHistory,
    },
};

pub mod pathfinding;
pub mod wfc;

pub struct EntiTilesAlgorithmPlugin;

impl Plugin for EntiTilesAlgorithmPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_type::<PathTile>()
            .register_type::<Pathfinder>()
            .register_type::<AsyncPathfinder>()
            .register_type::<Path>()
            .register_type::<PathNode>()
            .register_type::<PathGrid>();

        app.register_type::<AsyncWfcRunner>()
            .register_type::<WfcElement>()
            .register_type::<WfcHistory>();

        app.add_systems(
            Update,
            (
                pathfinding,
                pathfinding_async,
                wave_function_collapse,
                wave_function_collapse_async,
            ),
        );
    }
}

pub trait HeapElement {
    fn set_index(&mut self, index: usize);
    fn get_index(&self) -> usize;
}

#[derive(Debug, Clone, Reflect)]
pub struct LookupHeap<KHeap, KMap, V>
where
    KHeap: Ord + Copy + Debug,
    KMap: Eq + std::hash::Hash + Copy + Debug,
    V: Copy + HeapElement + Debug,
{
    pub count: usize,
    pub depth: usize,
    pub heap: Vec<Option<(KHeap, KMap)>>,
    pub lookup: HashMap<KMap, V>,
}

impl<KHeap, KMap, V> LookupHeap<KHeap, KMap, V>
where
    KHeap: Ord + Copy + Debug,
    KMap: Eq + std::hash::Hash + Copy + Debug,
    V: Copy + HeapElement + Debug,
{
    pub fn new() -> Self {
        Self {
            count: 0,
            depth: 1,
            heap: vec![None; 2],
            lookup: HashMap::default(),
        }
    }

    pub fn pop(&mut self) -> Option<V> {
        if self.count == 0 {
            return None;
        }

        let first = self.heap[1].unwrap();
        let last = self.heap[self.count].unwrap();

        self.lookup.get_mut(&last.1).unwrap().set_index(1);

        self.heap[1] = self.heap[self.count];
        self.heap[self.count] = None;

        self.count -= 1;

        self.shift_down(1);

        Some(self.lookup[&first.1])
    }

    pub fn heap_get(&self, key: KMap) -> Option<&(KHeap, KMap)> {
        if let Some(elem) = self.lookup.get(&key) {
            if let Some(h_elem) = self.heap.get(elem.get_index() as usize) {
                return h_elem.as_ref();
            }
        }

        None
    }

    pub fn insert_heap(&mut self, key_heap: KHeap, key_map: KMap) {
        if self.heap.len() == self.count + 1 {
            self.expand();
        }

        self.count += 1;
        let node = self.lookup.get_mut(&key_map).unwrap();
        node.set_index(self.count);
        self.heap[self.count] = Some((key_heap, key_map));
        self.shift_up(self.count);
    }

    #[inline]
    pub fn map_get(&self, key: &KMap) -> Option<&V> {
        self.lookup.get(key)
    }

    #[inline]
    pub fn map_get_mut(&mut self, key: &KMap) -> Option<&mut V> {
        self.lookup.get_mut(key)
    }

    #[inline]
    pub fn lookup_contains(&self, key: &KMap) -> bool {
        self.lookup.contains_key(key)
    }

    #[inline]
    pub fn update_lookup(&mut self, key: KMap, value: V) {
        self.lookup.insert(key, value);
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    fn expand(&mut self) {
        self.heap.extend_from_slice(&vec![None; self.depth * 2]);
        self.depth += 1;
    }

    fn shift_up(&mut self, index: usize) {
        let Some(mut this) = self.heap[index] else {
            return;
        };
        let Some(mut parent) = self.heap[index / 2] else {
            return;
        };

        while parent.0 > this.0 {
            let (swapped_this, _) = self.swap_node(this.1, parent.1);

            if swapped_this == 1 {
                break;
            } else {
                this = self.heap[swapped_this].unwrap();
                parent = self.heap[swapped_this / 2].unwrap();
            }
        }
    }

    fn shift_down(&mut self, index: usize) {
        if index * 2 >= self.count {
            return;
        };
        let Some(mut this) = self.heap[index] else {
            return;
        };
        let mut child = {
            let left = self.heap[index * 2].unwrap();
            let right = self.heap[index * 2 + 1].unwrap();
            if left.0 <= right.0 {
                left
            } else {
                right
            }
        };

        while child.0 < this.0 {
            let (swapped_this, _) = self.swap_node(this.1, child.1);

            if swapped_this * 2 > self.count {
                break;
            } else {
                this = self.heap[swapped_this].unwrap();
                child = {
                    let left = self.heap[swapped_this * 2].unwrap();
                    if let Some(right) = self.heap[swapped_this * 2 + 1] {
                        if left.0 < right.0 {
                            left
                        } else {
                            right
                        }
                    } else {
                        left
                    }
                };
            }
        }
    }

    /// Returns the heap_index after swap.
    /// (swapped_this_index, swapped_other_index)
    fn swap_node(&mut self, lhs_index: KMap, rhs_index: KMap) -> (usize, usize) {
        let lhs_heap_index = self.lookup.get(&lhs_index).unwrap().get_index();
        let rhs_heap_index = self.lookup.get(&rhs_index).unwrap().get_index();

        self.heap.swap(lhs_heap_index, rhs_heap_index);

        self.lookup
            .get_mut(&lhs_index)
            .unwrap()
            .set_index(rhs_heap_index);
        self.lookup
            .get_mut(&rhs_index)
            .unwrap()
            .set_index(lhs_heap_index);

        (rhs_heap_index, lhs_heap_index)
    }
}
