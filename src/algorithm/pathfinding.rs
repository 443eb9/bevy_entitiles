use bevy::{
    prelude::{Component, Entity, ParallelCommands, Plugin, Query, UVec2, Update},
    utils::{HashMap, HashSet},
};

use crate::{
    math::extension::ManhattanDistance,
    tilemap::Tilemap,
};

pub struct EntitilesPathfindingPlugin;

impl Plugin for EntitilesPathfindingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Update, pathfinding);
    }
}

pub struct PathPriorityQueue {
    weights: (u32, u32),
    layers: u32,
    nodes: Vec<Option<UVec2>>,
    indices: HashSet<UVec2>,
    node_counts: u32,
}

impl PathPriorityQueue {
    pub fn new(origin: UVec2, weights: (u32, u32)) -> Self {
        PathPriorityQueue {
            weights,
            layers: 1,
            indices: HashSet::new(),
            nodes: vec![None, Some(origin)],
            node_counts: 1,
        }
    }

    pub fn insert(&mut self, node: &PathNode, nodes_lookup: &HashMap<UVec2, PathNode>) {
        let mut current = 1;
        let cur_len = self.nodes.len();

        loop {
            if current > cur_len {
                self.nodes
                    .extend_from_slice(&vec![None; 2u32.pow(self.layers) as usize]);
                self.layers *= 2;
            }

            if let Some(cur_node) = self.nodes[current] {
                if node.weight(self.weights) < nodes_lookup[&cur_node].weight(self.weights) {
                    current *= 2;
                } else {
                    current = current * 2 + 1;
                }
            } else {
                self.nodes[current] = Some(node.index);
                self.indices.insert(node.index);
                break;
            }
        }

        self.node_counts += 1;
    }

    pub fn pop_min(&mut self) -> UVec2 {
        let mut current = 1;

        while self.nodes[current * 2].is_some() {
            current *= 2;
        }

        self.node_counts -= 1;
        let node = self.nodes[current].unwrap();
        self.indices.remove(&node);
        self.nodes[current] = None;
        node
    }

    pub fn is_empty(&self) -> bool {
        self.node_counts == 0
    }

    pub fn contains(&self, node_index: &UVec2) -> bool {
        self.indices.contains(node_index)
    }
}

#[derive(Component)]
pub struct PathTile {
    pub cost: u32,
}

#[derive(Component)]
pub struct Pathfinder {
    pub origin: UVec2,
    pub dest: UVec2,
    pub map_size: UVec2,
    pub allow_diagonal: bool,
    pub tilemap: Entity,
    pub custom_weight: Option<(u32, u32)>,
}

#[derive(Component, Default)]
pub struct Path {
    path: Vec<UVec2>,
    current_step: usize,
}

impl Path {
    pub fn step(&self) -> Option<UVec2> {
        if self.current_step >= self.path.len() {
            None
        } else {
            Some(self.path[self.current_step])
        }
    }
}

#[derive(Clone, Copy)]
pub struct PathNode {
    pub index: UVec2,
    pub parent: Option<UVec2>,
    pub g_cost: u32,
    pub h_cost: u32,
    pub g_cost_multiplier: u32,
}

impl PathNode {
    pub fn new(index: UVec2, g_cost: u32, g_cost_multiplier: u32, dest: UVec2) -> Self {
        PathNode {
            index,
            parent: None,
            g_cost,
            g_cost_multiplier,
            h_cost: dest.manhattan_distance(index),
        }
    }

    #[inline]
    pub fn weight(&self, weights: (u32, u32)) -> u32 {
        self.g_cost * weights.0 + self.h_cost * weights.1
    }

    pub fn neighbours(&self, tilemap: &Tilemap) -> [Option<(UVec2, Entity)>; 4] {
        let indices = [
            UVec2 {
                x: self.index.x + 1,
                y: self.index.y,
            },
            UVec2 {
                x: self.index.x - 1,
                y: self.index.y,
            },
            UVec2 {
                x: self.index.x,
                y: self.index.y + 1,
            },
            UVec2 {
                x: self.index.x,
                y: self.index.y - 1,
            },
        ];

        let mut result = [None; 4];
        for i in 0..4 {
            if let Some(entity) = tilemap.get(indices[i]) {
                result[0] = Some((indices[i], entity))
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
    finders
        .par_iter_mut()
        .for_each_mut(|(finder_entity, finder)| {
            let tilemap = &tilemaps_query.get(finder.tilemap).unwrap();
            let weights = finder.custom_weight.unwrap_or((1, 1));

            // check if origin or dest doesn't exists
            if tilemap.is_out_of_tilemap(finder.origin) || tilemap.is_out_of_tilemap(finder.dest) {
                complete_pathfinding(&commands, finder_entity, None);
                return;
            };

            // initialize containers
            // only path_records stores the actual node data
            // which acts as a lookup table
            // the others only store the index
            let origin_node = PathNode::new(finder.origin, 0, 0, finder.dest);
            let mut explored = HashSet::new();
            let mut to_explore = PathPriorityQueue::new(origin_node.index, weights);
            let mut path_records = HashMap::new();
            path_records.insert(origin_node.index, origin_node);
            to_explore.insert(&origin_node, &path_records);

            while !to_explore.is_empty() {
                let current = to_explore.pop_min();
                let cur_node = path_records[&current];

                if current == finder.dest {
                    let mut path = Path {
                        path: vec![],
                        current_step: 0,
                    };
                    let mut cur_index = current;
                    while cur_index != finder.origin {
                        path.path.push(cur_index);
                        cur_index = path_records[&cur_index].parent.unwrap();
                    }
                    complete_pathfinding(&commands, finder_entity, Some(path));
                }

                explored.insert(current);

                // explore neighbours
                for nei in cur_node.neighbours(&tilemap).into_iter() {
                    let Some((nei_index, nei_entity)) = nei else {
                        continue;
                    };

                    let nei_cost = tiles_query.get(nei_entity).unwrap().cost;

                    // update lookup
                    let _ = path_records.try_insert(
                        nei_index,
                        PathNode::new(nei_index, cur_node.g_cost + 1, nei_cost, origin_node.index),
                    );

                    let neighbour_tile = path_records.get(&nei_index).unwrap();
                    let already_scheduled = to_explore.contains(&neighbour_tile.index);

                    // if isn't on schedule or find a better path
                    if !already_scheduled || cur_node.g_cost + 1 < path_records[&nei_index].g_cost {
                        // update the new node
                        let mut new_node = PathNode::new(
                            nei_index,
                            cur_node.g_cost + 1,
                            nei_cost,
                            origin_node.index,
                        );
                        new_node.parent = Some(cur_node.index);
                        path_records.insert(nei_index, new_node);

                        if !already_scheduled {
                            to_explore.insert(&new_node, &path_records);
                        }
                    }
                }
            }
        });
}

pub fn complete_pathfinding(commands: &ParallelCommands, finder: Entity, path: Option<Path>) {
    commands.command_scope(|mut c| {
        let mut e = c.entity(finder);
        e.remove::<Pathfinder>();

        if let Some(path) = path {
            e.insert(path);
        }
    });
}
