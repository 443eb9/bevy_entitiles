use bevy::{
    ecs::query::Without,
    math::IVec2,
    prelude::{Component, Entity, ParallelCommands, Query},
    reflect::Reflect,
    utils::HashSet,
};

use crate::{
    math::extension::{ManhattanDistance, TileIndex},
    tilemap::{algorithm::path::PathTilemap, map::Tilemap},
};

use super::{HeapElement, LookupHeap};

#[derive(Component, Reflect)]
pub struct Pathfinder {
    pub origin: IVec2,
    pub dest: IVec2,
    pub allow_diagonal: bool,
    pub tilemap: Entity,
    pub custom_weight: Option<(u32, u32)>,
    pub max_step: Option<u32>,
}

#[derive(Component, Reflect)]
pub struct AsyncPathfinder {
    pub max_step_per_frame: u32,
}

#[derive(Component, Clone, Reflect)]
pub struct Path {
    path: Vec<IVec2>,
    current_step: usize,
    target_map: Entity,
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

    pub fn get_target_tilemap(&self) -> Entity {
        self.target_map
    }

    pub fn iter(&self) -> std::slice::Iter<IVec2> {
        self.path.iter()
    }
}

#[derive(Debug, Clone, Copy, Reflect)]
pub struct PathNode {
    pub index: IVec2,
    pub heap_index: usize,
    pub parent: Option<IVec2>,
    pub g_cost: u32,
    pub h_cost: u32,
    pub cost_to_pass: u32,
}

impl PathNode {
    pub fn new(
        index: IVec2,
        g_cost: u32,
        dest: IVec2,
        heap_index: usize,
        cost_to_pass: u32,
    ) -> Self {
        PathNode {
            index,
            heap_index,
            parent: None,
            g_cost,
            h_cost: dest.manhattan_distance(index),
            cost_to_pass,
        }
    }

    #[inline]
    pub fn weight(&self, weights: (u32, u32)) -> u32 {
        self.g_cost * weights.0 + self.h_cost * weights.1
    }
}

impl HeapElement for PathNode {
    #[inline]
    fn set_index(&mut self, index: usize) {
        self.heap_index = index
    }

    #[inline]
    fn get_index(&self) -> usize {
        self.heap_index
    }
}

#[derive(Component, Reflect)]
pub struct PathGrid {
    pub allow_diagonal: bool,
    pub dest: IVec2,
    pub weights: (u32, u32),
    pub lookup_heap: LookupHeap<u32, IVec2, PathNode>,
    pub explored: HashSet<IVec2>,
    pub steps: u32,
}

impl PathGrid {
    pub fn new(finder: &Pathfinder, root: &PathNode) -> Self {
        let weights = finder.custom_weight.unwrap_or((1, 1));
        let mut lookup_heap = LookupHeap::new();
        lookup_heap.update_lookup(root.index, *root);
        lookup_heap.insert_heap(root.weight(weights), root.index);
        PathGrid {
            allow_diagonal: finder.allow_diagonal,
            dest: finder.dest,
            weights,
            lookup_heap,
            explored: HashSet::new(),
            steps: 0,
        }
    }

    pub fn neighbours(
        &mut self,
        node: &PathNode,
        tilemap: &Tilemap,
        path_tilemap: &PathTilemap,
    ) -> Vec<IVec2> {
        node.index
            .neighbours(tilemap.tile_type, self.allow_diagonal)
            .into_iter()
            .filter_map(|n| self.get_or_register_new(n.unwrap(), self.dest, tilemap, path_tilemap))
            .collect()
    }

    #[inline]
    pub fn is_explored(&self, index: IVec2) -> bool {
        self.explored.contains(&index)
    }

    #[inline]
    pub fn is_scheduled(&self, index: IVec2) -> bool {
        if let Some(node) = self.lookup_heap.heap_get(index) {
            node.1 == index
        } else {
            false
        }
    }

    pub fn get(&self, index: IVec2) -> Option<&PathNode> {
        self.lookup_heap.map_get(&index)
    }

    pub fn get_mut(&mut self, index: IVec2) -> Option<&mut PathNode> {
        self.lookup_heap.map_get_mut(&index)
    }

    fn get_or_register_new(
        &mut self,
        index: IVec2,
        dest: IVec2,
        tilemap: &Tilemap,
        path_tilemap: &PathTilemap,
    ) -> Option<IVec2> {
        if tilemap.get(index).is_none() {
            return None;
        }

        if self.is_explored(index) {
            return None;
        }

        let Some(tile) = path_tilemap.get(index) else {
            return None;
        };

        if !self.lookup_heap.lookup_contains(&index) {
            self.lookup_heap
                .update_lookup(index, PathNode::new(index, 0, dest, 0, tile.cost));
        }

        Some(index)
    }

    pub fn schedule(&mut self, index: &IVec2) {
        let node = self.lookup_heap.map_get(index).unwrap();
        let key_heap = node.weight(self.weights);
        let key_map = node.index;
        self.lookup_heap.insert_heap(key_heap, key_map);
    }

    pub fn pop_closest(&mut self) -> Option<PathNode> {
        if let Some(min) = self.lookup_heap.pop() {
            self.explored.insert(min.index);
            Some(min)
        } else {
            None
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.lookup_heap.is_empty()
    }
}

pub fn pathfinding(
    commands: ParallelCommands,
    mut finders: Query<(Entity, &Pathfinder), Without<AsyncPathfinder>>,
    tilemaps_query: Query<(&Tilemap, &PathTilemap)>,
) {
    finders.par_iter_mut().for_each(|(finder_entity, finder)| {
        let Ok((tilemap, path_tilemap)) = tilemaps_query.get(finder.tilemap) else {
            panic!("Failed to find the tilemap! Did you add the tilemap and path tilemap to the same entity?");
        };

        // check if origin or dest doesn't exists
        if tilemap.get(finder.origin).is_none()
            || tilemap.get(finder.dest).is_none()
        {
            complete_pathfinding(&commands, finder_entity, None);
            return;
        };

        let origin_node = PathNode::new(finder.origin, 0, finder.dest, 1, 0);
        let mut path_grid = PathGrid::new(finder, &origin_node);
        find_path(
            &commands,
            &mut path_grid,
            finder_entity,
            finder,
            tilemap,
            path_tilemap,
            None,
        );
    });
}

pub fn pathfinding_async(
    commands: ParallelCommands,
    mut finders: Query<(Entity, &Pathfinder, &AsyncPathfinder, Option<&mut PathGrid>)>,
    tilemaps_query: Query<(&Tilemap, &PathTilemap)>,
) {
    finders
        .par_iter_mut()
        .for_each(|(finder_entity, finder, async_finder, path_grid)| {
            let (tilemap, path_tilemap) = tilemaps_query.get(finder.tilemap).unwrap();

            if let Some(mut grid) = path_grid {
                find_path(
                    &commands,
                    &mut grid,
                    finder_entity,
                    finder,
                    tilemap,
                    path_tilemap,
                    Some(async_finder),
                );
            } else {
                // check if origin or dest doesn't exists
                if tilemap.get(finder.origin).is_none() || tilemap.get(finder.dest).is_none() {
                    complete_pathfinding(&commands, finder_entity, None);
                    return;
                };
                let mut path_grid =
                    PathGrid::new(finder, &PathNode::new(finder.origin, 0, finder.dest, 1, 0));

                find_path(
                    &commands,
                    &mut path_grid,
                    finder_entity,
                    finder,
                    tilemap,
                    path_tilemap,
                    Some(async_finder),
                );

                commands.command_scope(|mut c| {
                    c.entity(finder_entity).insert(path_grid);
                });
            };
        });
}

pub fn complete_pathfinding(commands: &ParallelCommands, finder: Entity, path: Option<Path>) {
    #[cfg(feature = "debug")]
    if path.is_none() {
        println!("path not found");
    }

    commands.command_scope(|mut c| {
        let mut e = c.entity(finder);
        e.remove::<Pathfinder>();
        e.remove::<AsyncPathfinder>();

        if let Some(path) = path {
            e.insert(path);
        }
    });
}

fn find_path(
    commands: &ParallelCommands,
    path_grid: &mut PathGrid,
    finder_entity: Entity,
    finder: &Pathfinder,
    tilemap: &Tilemap,
    path_tilemap: &PathTilemap,
    async_finder: Option<&AsyncPathfinder>,
) {
    let mut frame_step = 0;
    let max_frame_step = {
        if let Some(async_finder) = async_finder {
            async_finder.max_step_per_frame
        } else {
            u32::MAX
        }
    };

    while !path_grid.is_empty() {
        path_grid.steps += 1;
        frame_step += 1;
        if path_grid.steps > finder.max_step.unwrap_or(u32::MAX) {
            break;
        }
        if frame_step >= max_frame_step {
            return;
        }

        let mut current = path_grid.pop_closest().unwrap();

        if current.index == finder.dest {
            let mut path = Path {
                path: vec![],
                current_step: 0,
                target_map: finder.tilemap,
            };
            while current.index != finder.origin {
                path.path.push(current.index);
                current = *path_grid.get(current.parent.unwrap()).unwrap();
            }

            complete_pathfinding(&commands, finder_entity, Some(path));
            return;
        }

        let neighbours = path_grid.neighbours(&current, tilemap, &path_tilemap);

        // explore neighbours
        for neighbour in neighbours {
            let already_scheduled = path_grid.is_scheduled(neighbour);
            let neighbour_node = path_grid.get_mut(neighbour).unwrap();

            // if isn't on schedule or find a better path
            if !already_scheduled || current.g_cost < neighbour_node.g_cost {
                // update the new node
                neighbour_node.g_cost = current.g_cost + neighbour_node.cost_to_pass;
                neighbour_node.parent = Some(current.index);

                if !already_scheduled {
                    path_grid.schedule(&neighbour);
                }
            }
        }
    }

    complete_pathfinding(&commands, finder_entity, None);
}
