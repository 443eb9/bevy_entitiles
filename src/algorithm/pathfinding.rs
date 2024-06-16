use std::{cmp::Ordering, collections::BinaryHeap};

use bevy::{
    ecs::{
        entity::EntityHashMap,
        system::{Commands, Query, Res, Resource},
    },
    math::IVec2,
    prelude::{Component, Entity},
    reflect::Reflect,
    utils::{Entry, HashMap, HashSet},
};

use crate::{
    math::ext::{ManhattanDistance, TileIndex},
    tilemap::{algorithm::path::PathTilemap, map::TilemapType},
};

#[cfg(feature = "multi-threaded")]
use bevy::tasks::{AsyncComputeTaskPool, Task};
#[cfg(feature = "multi-threaded")]
use std::sync::{Arc, Mutex, MutexGuard};

#[derive(Resource, Default)]
pub struct PathTilemaps {
    #[cfg(feature = "multi-threaded")]
    pub(crate) tilemaps: EntityHashMap<Arc<Mutex<PathTilemap>>>,
    #[cfg(not(feature = "multi-threaded"))]
    pub(crate) tilemaps: EntityHashMap<PathTilemap>,
}

#[cfg(feature = "multi-threaded")]
impl PathTilemaps {
    #[inline]
    pub fn get(&self, tilemap: Entity) -> Option<Arc<Mutex<PathTilemap>>> {
        self.tilemaps.get(&tilemap).cloned()
    }

    #[inline]
    pub fn lock(&self, tilemap: Entity) -> Option<MutexGuard<PathTilemap>> {
        self.tilemaps.get(&tilemap).map(|t| t.lock().unwrap())
    }

    #[inline]
    pub fn get_mut(&mut self, tilemap: Entity) -> Option<&mut Arc<Mutex<PathTilemap>>> {
        self.tilemaps.get_mut(&tilemap)
    }

    #[inline]
    pub fn insert(&mut self, tilemap: Entity, path_tilemap: PathTilemap) {
        self.tilemaps
            .insert(tilemap, Arc::new(Mutex::new(path_tilemap)));
    }

    #[inline]
    pub fn remove(&mut self, tilemap: Entity) {
        self.tilemaps.remove(&tilemap);
    }
}

#[cfg(not(feature = "multi-threaded"))]
impl PathTilemaps {
    #[inline]
    pub fn get(&self, tilemap: Entity) -> Option<&PathTilemap> {
        self.tilemaps.get(&tilemap)
    }

    #[inline]
    pub fn get_mut(&mut self, tilemap: Entity) -> Option<&mut PathTilemap> {
        self.tilemaps.get_mut(&tilemap)
    }

    #[inline]
    pub fn insert(&mut self, tilemap: Entity, path_tilemap: PathTilemap) {
        self.tilemaps.insert(tilemap, path_tilemap);
    }

    #[inline]
    pub fn remove(&mut self, tilemap: Entity) {
        self.tilemaps.remove(&tilemap);
    }
}

#[derive(Component, Reflect)]
pub struct PathFinder {
    pub origin: IVec2,
    pub dest: IVec2,
    pub allow_diagonal: bool,
    pub max_steps: Option<u32>,
    #[cfg(not(feature = "multi-threaded"))]
    pub max_steps_per_frame: u32,
    #[cfg(not(feature = "multi-threaded"))]
    pub tilemap_ty: TilemapType,
}

#[derive(Component)]
pub struct PathFindingQueue {
    pub(crate) finders: EntityHashMap<PathFinder>,
    #[cfg(feature = "multi-threaded")]
    pub(crate) tasks: EntityHashMap<Task<Path>>,
}

impl PathFindingQueue {
    pub fn new_with_schedules(schedules: impl Iterator<Item = (Entity, PathFinder)>) -> Self {
        PathFindingQueue {
            finders: schedules.collect(),
            #[cfg(feature = "multi-threaded")]
            tasks: EntityHashMap::default(),
        }
    }

    #[cfg(feature = "multi-threaded")]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    #[inline]
    pub fn schedule(&mut self, requester: Entity, pathfinder: PathFinder) {
        self.finders.insert(requester, pathfinder);
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

#[cfg_attr(not(feature = "multi-threaded"), derive(Component))]
pub struct PathGrid {
    pub requester: Entity,
    pub tilemap: Entity,
    pub allow_diagonal: bool,
    pub tilemap_ty: TilemapType,
    pub origin: IVec2,
    pub dest: IVec2,
    pub to_explore: BinaryHeap<PathNode>,
    pub explored: HashSet<IVec2>,
    pub all_nodes: HashMap<IVec2, PathNode>,
    pub steps: u32,
    pub max_steps: Option<u32>,
    #[cfg(feature = "multi-threaded")]
    pub path_tilemap: Arc<Mutex<PathTilemap>>,
    #[cfg(not(feature = "multi-threaded"))]
    pub max_steps_per_frame: u32,
    #[cfg(not(feature = "multi-threaded"))]
    pub is_done: bool,
}

impl PathGrid {
    pub fn new(
        finder: PathFinder,
        requester: Entity,
        tilemap: Entity,
        tilemap_ty: TilemapType,
        #[cfg(feature = "multi-threaded")] path_tilemap: Arc<Mutex<PathTilemap>>,
    ) -> Self {
        PathGrid {
            requester,
            tilemap,
            allow_diagonal: finder.allow_diagonal,
            tilemap_ty,
            origin: finder.origin,
            dest: finder.dest,
            to_explore: BinaryHeap::new(),
            explored: HashSet::new(),
            all_nodes: HashMap::new(),
            steps: 0,
            max_steps: finder.max_steps,
            #[cfg(feature = "multi-threaded")]
            path_tilemap,
            #[cfg(not(feature = "multi-threaded"))]
            max_steps_per_frame: finder.max_steps_per_frame,
            #[cfg(not(feature = "multi-threaded"))]
            is_done: false,
        }
    }

    #[cfg(feature = "multi-threaded")]
    pub fn get_or_register(&mut self, index: IVec2) -> Option<PathNode> {
        if let Some(node) = self.all_nodes.get(&index) {
            Some(node.clone())
        } else {
            self.path_tilemap.lock().unwrap().get(index).map(|tile| {
                let new = PathNode::new(index, u32::MAX, self.dest, tile.cost);
                self.all_nodes.insert(index, new);
                new
            })
        }
    }

    #[cfg(not(feature = "multi-threaded"))]
    pub fn get_or_register(
        &mut self,
        index: IVec2,
        path_tilemaps: &PathTilemaps,
    ) -> Option<PathNode> {
        if let Some(node) = self.all_nodes.get(&index) {
            Some(*node)
        } else {
            path_tilemaps
                .get(self.tilemap)
                .unwrap()
                .get(index)
                .map(|tile| {
                    let new = PathNode::new(index, u32::MAX, self.dest, tile.cost);
                    self.all_nodes.insert(index, new);
                    new
                })
        }
    }

    #[cfg(feature = "multi-threaded")]
    pub fn neighbours(&mut self, index: IVec2) -> Vec<PathNode> {
        index
            .neighbours(self.tilemap_ty, self.allow_diagonal)
            .into_iter()
            .filter_map(|p| p.and_then(|p| self.get_or_register(p)))
            .collect()
    }

    #[cfg(not(feature = "multi-threaded"))]
    pub fn neighbours(&mut self, index: IVec2, path_tilemaps: &PathTilemaps) -> Vec<PathNode> {
        index
            .neighbours(self.tilemap_ty, self.allow_diagonal)
            .into_iter()
            .filter_map(|p| p.and_then(|p| self.get_or_register(p, path_tilemaps)))
            .collect()
    }

    #[allow(unused)]
    pub fn find_path(&mut self, path_tilemaps: Option<&PathTilemaps>) {
        let origin = PathNode::new(self.origin, 0, self.dest, 0);
        self.to_explore.push(origin.clone());
        self.all_nodes.insert(self.origin, origin);

        #[cfg(not(feature = "multi-threaded"))]
        let mut steps_cur_frame = 0;

        while !self.to_explore.is_empty() {
            if let Some(max_steps) = self.max_steps {
                if self.steps > max_steps {
                    return;
                }
            }
            self.steps += 1;

            #[cfg(not(feature = "multi-threaded"))]
            {
                if steps_cur_frame >= self.max_steps_per_frame {
                    return;
                }
                steps_cur_frame += 1;
            }

            let current = self.to_explore.pop().unwrap();
            if current.index == self.dest {
                break;
            }
            if current.g_cost > self.all_nodes[&current.index].g_cost {
                continue;
            }

            #[cfg(feature = "multi-threaded")]
            let neighbours = self.neighbours(current.index);
            #[cfg(not(feature = "multi-threaded"))]
            let neighbours = self.neighbours(current.index, path_tilemaps.unwrap());

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

        #[cfg(not(feature = "multi-threaded"))]
        {
            self.is_done = true;
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

#[cfg(feature = "multi-threaded")]
pub fn pathfinding_scheduler(
    mut queues_query: Query<(Entity, &TilemapType, &mut PathFindingQueue)>,
    path_tilemaps: Res<PathTilemaps>,
) {
    let thread_pool = AsyncComputeTaskPool::get();
    queues_query
        .iter_mut()
        .for_each(|(tilemap, ty, mut queue)| {
            let mut tasks = Vec::new();
            let path_tilemap = path_tilemaps.get(tilemap).unwrap();
            queue.finders.drain().for_each(|(requester, finder)| {
                let ty = *ty;
                let path_tilemap = path_tilemap.clone();
                let task = thread_pool.spawn(async move {
                    let mut grid = PathGrid::new(finder, requester, tilemap, ty, path_tilemap);
                    grid.find_path(None);
                    grid.collect_path()
                });
                tasks.push((requester, task));
            });
            queue.tasks.extend(tasks);
        });
}

#[cfg(not(feature = "multi-threaded"))]
pub fn pathfinding_scheduler(
    mut commands: Commands,
    mut queues_query: Query<(Entity, &TilemapType, &mut PathFindingQueue)>,
) {
    queues_query
        .iter_mut()
        .for_each(|(tilemap, ty, mut queue)| {
            queue.finders.drain().for_each(|(requester, finder)| {
                commands
                    .entity(requester)
                    .insert(PathGrid::new(finder, requester, tilemap, *ty));
            });
        });
}

#[cfg(not(feature = "multi-threaded"))]
pub fn path_finding_single_threaded(
    mut commands: Commands,
    mut tasks_query: Query<(Entity, &mut PathGrid)>,
    path_tilemaps: Res<PathTilemaps>,
) {
    let Some((requester, mut cur_task)) = tasks_query.iter_mut().next() else {
        return;
    };

    cur_task.find_path(Some(&path_tilemaps));
    if cur_task.is_done {
        commands.entity(requester).insert(cur_task.collect_path());
        commands.entity(requester).remove::<PathGrid>();
    }
}

#[cfg(feature = "multi-threaded")]
pub fn path_assigner(mut commands: Commands, mut queues_query: Query<&mut PathFindingQueue>) {
    queues_query.iter_mut().for_each(|mut queue| {
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
