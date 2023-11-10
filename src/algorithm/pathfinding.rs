use bevy::{
    prelude::{Component, Entity, ParallelCommands, Plugin, Query, ResMut, UVec2, Update},
    utils::{HashMap, HashSet},
};

use crate::{debug::common::DebugResource, math::extension::ManhattanDistance, tilemap::Tilemap};

pub struct EntitilesPathfindingPlugin;

impl Plugin for EntitilesPathfindingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Update, pathfinding);
    }
}

struct PathCache {
    pub depth: usize,
    pub count: usize,
    pub weights: (u32, u32),
    pub nodes: Vec<Option<(u32, UVec2)>>,
}

impl PathCache {
    pub fn new(weights: (u32, u32), root: &PathNode) -> Self {
        PathCache {
            depth: 1,
            count: 1,
            weights,
            nodes: Vec::from([None, Some((root.weight(weights), root.index))]),
        }
    }

    pub fn insert(&mut self, node: &PathNode) {
        if self.nodes.len() == self.count + 1 {
            self.expand();
        }

        self.count += 1;
        self.nodes[self.count] = Some((node.weight(self.weights), node.index));
        self.shift_up(self.count);
    }

    pub fn pop_min(&mut self) -> Option<UVec2> {
        if self.count == 0 {
            return None;
        }

        let result = self.nodes[1].unwrap().1;
        self.count -= 1;
        self.nodes[1] = self.nodes[self.count];
        self.shift_down(1);
        Some(result)
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn contains(&self, node_index: &UVec2) -> bool {
        for index in self.nodes.iter() {
            if let Some((_, node)) = index {
                if node == node_index {
                    return true;
                }
            }
        }
        false
    }

    fn expand(&mut self) {
        self.nodes.extend_from_slice(&vec![None; self.depth * 2]);
        self.depth += 1;
    }

    fn shift_up(&mut self, index: usize) {
        let mut i = index;
        while i > 1 && self.nodes[i / 2].unwrap().0 > self.nodes[i].unwrap().0 {
            self.nodes.swap(i / 2, i);
            i /= 2;
        }
    }

    fn shift_down(&mut self, index: usize) {
        let mut i = index;
        while i * 2 <= self.count {
            let mut j = i * 2;
            if j < self.count && self.nodes[j].unwrap().0 > self.nodes[j + 1].unwrap().0 {
                j += 1;
            }
            if self.nodes[i].unwrap().0 <= self.nodes[j].unwrap().0 {
                break;
            }
            self.nodes.swap(i, j);
            i = j;
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct PathTile {
    pub cost: u32,
}

#[derive(Component)]
pub struct Pathfinder {
    pub origin: UVec2,
    pub dest: UVec2,
    pub allow_diagonal: bool,
    pub tilemap: Entity,
    pub custom_weight: Option<(u32, u32)>,
}

#[derive(Component, Clone)]
pub struct Path {
    path: Vec<UVec2>,
    current_step: usize,
    target_map: Entity,
}

impl Path {
    pub fn step(&self) -> Option<UVec2> {
        if self.current_step >= self.path.len() {
            None
        } else {
            Some(self.path[self.current_step])
        }
    }

    pub fn get_target_tilemap(&self) -> Entity {
        self.target_map
    }

    pub fn iter(&self) -> std::slice::Iter<UVec2> {
        self.path.iter()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PathNode {
    pub index: UVec2,
    pub parent: Option<UVec2>,
    pub g_cost: u32,
    pub h_cost: u32,
}

impl PathNode {
    pub fn new(index: UVec2, g_cost: u32, dest: UVec2) -> Self {
        PathNode {
            index,
            parent: None,
            g_cost,
            h_cost: dest.manhattan_distance(index),
        }
    }

    #[inline]
    pub fn weight(&self, weights: (u32, u32)) -> u32 {
        self.g_cost * weights.0 + self.h_cost * weights.1
    }

    pub fn neighbours_no_diag(&self, tilemap: &Tilemap) -> Vec<(UVec2, Entity)> {
        let mut indices = Vec::with_capacity(4);
        if self.index.x > 0 {
            indices.push(UVec2 {
                x: self.index.x - 1,
                y: self.index.y,
            });
        }
        if self.index.y > 0 {
            indices.push(UVec2 {
                x: self.index.x,
                y: self.index.y - 1,
            });
        }
        if self.index.x < tilemap.size.x - 1 {
            indices.push(UVec2 {
                x: self.index.x + 1,
                y: self.index.y,
            });
        }
        if self.index.y < tilemap.size.y - 1 {
            indices.push(UVec2 {
                x: self.index.x,
                y: self.index.y + 1,
            });
        }

        let mut result = Vec::with_capacity(4);
        for i in 0..indices.len() {
            if let Some(entity) = tilemap.get(indices[i]) {
                result.push((indices[i], entity));
            }
        }
        result
    }

    pub fn neighbours_diag(&self, tilemap: &Tilemap) -> Vec<(UVec2, Entity)> {
        let mut indices = Vec::with_capacity(8);
        if self.index.x > 0 {
            indices.push(UVec2 {
                x: self.index.x - 1,
                y: self.index.y,
            });
        }
        if self.index.y > 0 {
            indices.push(UVec2 {
                x: self.index.x,
                y: self.index.y - 1,
            });
        }
        if self.index.x < tilemap.size.x - 1 {
            indices.push(UVec2 {
                x: self.index.x + 1,
                y: self.index.y,
            });
        }
        if self.index.y < tilemap.size.y - 1 {
            indices.push(UVec2 {
                x: self.index.x,
                y: self.index.y + 1,
            });
        }
        if self.index.x > 0 && self.index.y > 0 {
            indices.push(UVec2 {
                x: self.index.x - 1,
                y: self.index.y - 1,
            });
        }
        if self.index.x < tilemap.size.x - 1 && self.index.y > 0 {
            indices.push(UVec2 {
                x: self.index.x + 1,
                y: self.index.y - 1,
            });
        }
        if self.index.x > 0 && self.index.y < tilemap.size.y - 1 {
            indices.push(UVec2 {
                x: self.index.x - 1,
                y: self.index.y + 1,
            });
        }
        if self.index.x < tilemap.size.x - 1 && self.index.y < tilemap.size.y - 1 {
            indices.push(UVec2 {
                x: self.index.x + 1,
                y: self.index.y + 1,
            });
        }

        let mut result = Vec::with_capacity(8);
        for i in 0..indices.len() {
            if let Some(entity) = tilemap.get(indices[i]) {
                result.push((indices[i], entity));
            }
        }
        result
    }
}

pub fn pathfinding(
    commands: ParallelCommands,
    mut finders: Query<(Entity, &Pathfinder)>,
    tilemaps_query: Query<&Tilemap>,
    tiles_query: Query<&PathTile>,
) {
    finders.par_iter_mut().for_each_mut(|(finder_entity, finder)| {
        #[cfg(feature = "debug")]
        println!("pathfinding start! {} -> {}", finder.origin, finder.dest);
        let tilemap = &tilemaps_query.get(finder.tilemap).unwrap();
        let weights = finder.custom_weight.unwrap_or((1, 1));

        // check if origin or dest doesn't exists
        if tilemap.is_out_of_tilemap(finder.origin) || tilemap.is_out_of_tilemap(finder.dest) {
            #[cfg(feature = "debug")]
            println!("out of tilemap");
            complete_pathfinding(&commands, finder_entity, None);
            return;
        };

        // initialize containers
        // only path_records stores the actual node data
        // which acts as a lookup table
        // the others only store the index
        let origin_node = PathNode::new(finder.origin, 0, finder.dest);
        let mut explored = HashSet::new();
        let mut to_explore = PathCache::new(weights, &origin_node);
        let mut path_records = HashMap::new();
        path_records.insert(origin_node.index, origin_node);
        to_explore.insert(&origin_node);

        #[cfg(feature = "debug")]
        let mut i = 0;

        while !to_explore.is_empty() {
            #[cfg(feature = "debug")]
            {
                i += 1;
            }

            let current = to_explore.pop_min().unwrap();
            let cur_node = path_records[&current];

            if current == finder.dest {
                let mut path = Path {
                    path: vec![],
                    current_step: 0,
                    target_map: finder.tilemap,
                };
                let mut cur_index = current;
                while cur_index != finder.origin {
                    path.path.push(cur_index);
                    cur_index = path_records[&cur_index].parent.unwrap();
                }

                #[cfg(feature = "debug")]
                println!(
                    "pathfinding finished! after {} steps, length = {}",
                    i,
                    path.path.len()
                );
                complete_pathfinding(&commands, finder_entity, Some(path));
                return;
            }

            explored.insert(current);

            let neighbours = {
                if finder.allow_diagonal {
                    cur_node.neighbours_diag(&tilemap)
                } else {
                    cur_node.neighbours_no_diag(&tilemap)
                }
            };

            // explore neighbours
            for nei in neighbours.into_iter() {
                let (nei_index, nei_entity) = nei;
                let Ok(nei_cost) = tiles_query.get(nei_entity) else {
                    continue;
                };

                if explored.contains(&nei_index) {
                    continue;
                }

                // update lookup
                let _ = path_records.try_insert(
                    nei_index,
                    PathNode::new(nei_index, cur_node.g_cost + nei_cost.cost, finder.dest),
                );

                let neighbour_tile = path_records.get(&nei_index).unwrap();
                let already_scheduled = to_explore.contains(&neighbour_tile.index);

                // if isn't on schedule or find a better path
                if !already_scheduled || cur_node.g_cost < path_records[&nei_index].g_cost {
                    // update the new node
                    let mut new_node =
                        PathNode::new(nei_index, cur_node.g_cost + nei_cost.cost, finder.dest);
                    new_node.parent = Some(cur_node.index);
                    path_records.insert(nei_index, new_node);

                    if !already_scheduled {
                        to_explore.insert(&new_node);
                    }
                }
            }
        }

        complete_pathfinding(&commands, finder_entity, None);
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

        if let Some(path) = path {
            e.insert(path);
        }
    });
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_priority_queue() {
        let origin = PathNode {
            index: UVec2::ZERO,
            parent: None,
            g_cost: 0,
            h_cost: 0,
        };
        let mut queue = PathCache::new((1, 1), &origin);

        let node1 = PathNode::new(UVec2::ZERO, 1, UVec2::new(0, 0));
        let node2 = PathNode::new(UVec2::ZERO, 2, UVec2::new(0, 0));
        let node3 = PathNode::new(UVec2::ZERO, 3, UVec2::new(0, 0));
        let node4 = PathNode::new(UVec2::ZERO, 4, UVec2::new(0, 0));
        let node5 = PathNode::new(UVec2::ZERO, 5, UVec2::new(0, 0));
        let node6 = PathNode::new(UVec2::ZERO, 6, UVec2::new(0, 0));

        queue.insert(&node4);
        queue.insert(&node3);
        queue.insert(&node5);
        queue.insert(&node6);
        queue.insert(&node2);
        queue.insert(&node1);

        for _ in 0..6 {
            println!("{:?}", queue.pop_min());
        }
    }
}
