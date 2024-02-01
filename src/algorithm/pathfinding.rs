use std::{cmp::Ordering, collections::BinaryHeap, sync::Arc};

use bevy::{
    ecs::system::{Commands, Query},
    math::IVec2,
    prelude::{Component, Entity},
    reflect::Reflect,
    tasks::{AsyncComputeTaskPool, Task},
    utils::{EntityHashMap, Entry, HashMap, HashSet},
};

use crate::{
    math::extension::{ManhattanDistance, TileIndex},
    tilemap::{algorithm::path::PathTilemap, map::TilemapType},
};

#[derive(Component, Reflect)]
pub struct PathFinder {
    pub origin: IVec2,
    pub dest: IVec2,
    pub allow_diagonal: bool,
    pub max_steps: Option<u32>,
}

#[derive(Component)]
pub struct PathFindingQueue {
    pub(crate) finders: EntityHashMap<Entity, PathFinder>,
    pub(crate) tasks: EntityHashMap<Entity, Task<Path>>,
    pub(crate) cache: Arc<PathTilemap>,
}

impl PathFindingQueue {
    pub fn new(cache: PathTilemap) -> Self {
        PathFindingQueue {
            finders: EntityHashMap::default(),
            tasks: EntityHashMap::default(),
            cache: Arc::new(cache),
        }
    }

    pub fn new_with_schedules(
        cache: PathTilemap,
        schedules: impl Iterator<Item = (Entity, PathFinder)>,
    ) -> Self {
        PathFindingQueue {
            finders: schedules.collect(),
            tasks: EntityHashMap::default(),
            cache: Arc::new(cache),
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    #[inline]
    pub fn schedule(&mut self, requester: Entity, pathfinder: PathFinder) {
        self.finders.insert(requester, pathfinder);
    }

    #[inline]
    pub fn get_cache(&self) -> Arc<PathTilemap> {
        self.cache.clone()
    }

    #[inline]
    pub fn get_cache_mut(&mut self) -> &mut PathTilemap {
        Arc::get_mut(&mut self.cache).unwrap()
    }
}

#[derive(Component, Clone, Reflect)]
pub struct Path {
    path: Vec<IVec2>,
    current_step: usize,
    tilemap: Entity,
}

impl Path {
    /// Step to next target. Or do nothing if already arrived.
    pub fn step(&mut self) {
        if self.current_step >= self.path.len() {
            return;
        }
        self.current_step += 1;
    }

    /// Get the current target.
    pub fn cur_target(&self) -> IVec2 {
        self.path[self.current_step]
    }

    /// Return is arrived.
    pub fn is_arrived(&self) -> bool {
        self.current_step >= self.path.len()
    }

    pub fn tilemap(&self) -> Entity {
        self.tilemap
    }

    pub fn iter(&self) -> std::slice::Iter<IVec2> {
        self.path.iter()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PathNode {
    pub index: IVec2,
    pub parent: Option<IVec2>,
    pub g_cost: u32,
    pub h_cost: u32,
    pub cost_to_pass: u32,
}

impl PartialOrd for PathNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PathNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .g_cost
            .cmp(&self.g_cost)
            .then(other.h_cost.cmp(&self.h_cost))
    }
}

impl PathNode {
    pub fn new(index: IVec2, g_cost: u32, dest: IVec2, cost_to_pass: u32) -> Self {
        PathNode {
            index,
            parent: None,
            g_cost,
            h_cost: dest.manhattan_distance(index),
            cost_to_pass,
        }
    }

    #[inline]
    pub fn weight(&self) -> u32 {
        self.g_cost + self.h_cost
    }
}

pub struct PathGrid {
    pub requester: Entity,
    pub tilemap: Entity,
    pub allow_diagonal: bool,
    pub origin: IVec2,
    pub dest: IVec2,
    pub to_explore: BinaryHeap<PathNode>,
    pub explored: HashSet<IVec2>,
    pub all_nodes: HashMap<IVec2, PathNode>,
    pub steps: u32,
    pub max_steps: Option<u32>,
    pub path_tilemap: Arc<PathTilemap>,
}

impl PathGrid {
    pub fn new(
        finder: PathFinder,
        requester: Entity,
        tilemap: Entity,
        path_tilemap: Arc<PathTilemap>,
    ) -> Self {
        PathGrid {
            requester,
            tilemap,
            allow_diagonal: finder.allow_diagonal,
            origin: finder.origin,
            dest: finder.dest,
            to_explore: BinaryHeap::new(),
            explored: HashSet::new(),
            all_nodes: HashMap::new(),
            steps: 0,
            max_steps: finder.max_steps,
            path_tilemap,
        }
    }

    pub fn get_or_register(&mut self, index: IVec2) -> Option<PathNode> {
        if let Some(node) = self.all_nodes.get(&index) {
            Some(node.clone())
        } else {
            self.path_tilemap.get(index).map(|tile| {
                let new = PathNode::new(index, u32::MAX, self.dest, tile.cost);
                self.all_nodes.insert(index, new);
                new
            })
        }
    }

    pub fn neighbours(&mut self, index: IVec2, ty: TilemapType) -> Vec<PathNode> {
        index
            .neighbours(ty, self.allow_diagonal)
            .into_iter()
            .filter_map(|p| p.and_then(|p| self.get_or_register(p)))
            .collect()
    }

    pub fn find_path(&mut self, ty: TilemapType) {
        let origin = PathNode::new(self.origin, 0, self.dest, 0);
        self.to_explore.push(origin.clone());
        self.all_nodes.insert(self.origin, origin);

        while !self.to_explore.is_empty() {
            if let Some(max_steps) = self.max_steps {
                if self.steps > max_steps {
                    break;
                }
            }
            self.steps += 1;

            let current = self.to_explore.pop().unwrap();
            if current.index == self.dest {
                return;
            }
            if current.g_cost > self.all_nodes[&current.index].g_cost {
                continue;
            }

            let neighbours = self.neighbours(current.index, ty);

            for mut neighbour in neighbours {
                neighbour.g_cost = current.g_cost + neighbour.cost_to_pass;
                neighbour.parent = Some(current.index);

                match self.all_nodes.entry(neighbour.index) {
                    Entry::Occupied(mut e) => {
                        if e.get().g_cost > neighbour.g_cost {
                            e.insert(neighbour.clone());
                            self.to_explore.push(neighbour);
                        }
                    }
                    Entry::Vacant(e) => {
                        e.insert(neighbour.clone());
                        self.to_explore.push(neighbour);
                    }
                };
            }
        }
    }

    pub fn collect_path(&self) -> Path {
        let mut path = Path {
            path: vec![],
            current_step: 0,
            tilemap: self.tilemap,
        };
        let mut current = self.all_nodes.get(&self.dest).unwrap();
        while current.index != self.origin {
            path.path.push(current.index);
            current = self.all_nodes.get(&current.parent.unwrap()).unwrap();
        }
        path
    }
}

pub fn pathfinding_scheduler(
    mut queues_query: Query<(Entity, &TilemapType, &mut PathFindingQueue)>,
) {
    let thread_pool = AsyncComputeTaskPool::get();
    queues_query.for_each_mut(|(tilemap, ty, mut queue)| {
        let mut tasks = Vec::new();
        let path_tilemap = queue.cache.clone();
        queue.finders.drain().for_each(|(requester, finder)| {
            let ty = *ty;
            let path_tilemap = path_tilemap.clone();
            let task = thread_pool.spawn(async move {
                let mut grid = PathGrid::new(finder, requester, tilemap, path_tilemap.clone());
                grid.find_path(ty);
                grid.collect_path()
            });
            tasks.push((requester, task));
        });
        queue.tasks.extend(tasks);
    });
}

pub fn path_assigner(mut commands: Commands, mut queues_query: Query<&mut PathFindingQueue>) {
    queues_query.for_each_mut(|mut queue| {
        let mut completed = Vec::new();
        queue.tasks.iter_mut().for_each(|(requester, task)| {
            if let Some(path) = bevy::tasks::block_on(futures_lite::future::poll_once(task)) {
                commands.entity(*requester).insert(path);
                completed.push(*requester);
            }
        });
        completed.iter().for_each(|requester| {
            queue.tasks.remove(requester);
        });
    });
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::tilemap::algorithm::path::PathTile;

    #[test]
    fn test_pathfinding() {
        let mut path_tilemap = PathTilemap::new();
        for y in 0..=3 {
            for x in 0..=3 {
                path_tilemap.set(
                    IVec2 { x, y },
                    PathTile {
                        cost: rand::random::<u32>() % 10,
                    },
                );
            }
        }

        let mut grid = PathGrid {
            tilemap: Entity::PLACEHOLDER,
            requester: Entity::PLACEHOLDER,
            allow_diagonal: false,
            origin: IVec2::ZERO,
            dest: IVec2::new(3, 3),
            to_explore: BinaryHeap::new(),
            explored: HashSet::new(),
            all_nodes: HashMap::new(),
            steps: 0,
            max_steps: None,
            path_tilemap: Arc::new(path_tilemap),
        };

        grid.find_path(TilemapType::Square);
        let path = grid.collect_path();
        dbg!(path.path);
    }
}
