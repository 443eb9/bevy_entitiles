use std::mem::swap;

use bevy::{
    prelude::{Component, Entity, IVec2, ParallelCommands, Plugin, Query, ResMut, UVec2, Update},
    utils::{hashbrown::HashMap, HashSet},
};

use crate::{debug::common::DebugResource, math::extension::ManhattanDistance, tilemap::Tilemap};

pub struct EntitilesPathfindingPlugin;

impl Plugin for EntitilesPathfindingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Update, pathfinding);
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
    pub heap_index: usize,
    pub parent: Option<UVec2>,
    pub g_cost: u32,
    pub h_cost: u32,
    pub cost_to_pass: u32,
}

impl PathNode {
    pub fn new(
        index: UVec2,
        g_cost: u32,
        dest: UVec2,
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

struct PathGrid {
    pub allow_diagonal: bool,
    pub dest: UVec2,
    pub depth: usize,
    pub count: usize,
    pub weights: (u32, u32),
    pub index_to_path_node: HashMap<UVec2, PathNode>,
    pub to_explore: Vec<Option<(u32, UVec2)>>,
    pub explored: HashSet<UVec2>,
}

impl PathGrid {
    pub fn new(finder: &Pathfinder, root: &PathNode) -> Self {
        let weights = finder.custom_weight.unwrap_or((1, 1));
        PathGrid {
            allow_diagonal: finder.allow_diagonal,
            dest: finder.dest,
            depth: 1,
            count: 1,
            weights,
            index_to_path_node: HashMap::from([(root.index, root.clone())]),
            to_explore: Vec::from([None, Some((root.weight(weights), root.index))]),
            explored: HashSet::new(),
        }
    }

    pub fn neighbours(
        &mut self,
        node: &PathNode,
        tilemap: &Tilemap,
        path_tiles_query: &Query<&PathTile>,
    ) -> Vec<UVec2> {
        let count = {
            if self.allow_diagonal {
                8
            } else {
                4
            }
        };

        let mut result = Vec::with_capacity(count);
        for dy in [-1, 0, 1] {
            for dx in [-1, 0, 1] {
                if dx == 0 && dy == 0 {
                    continue;
                }

                if !self.allow_diagonal && dx != 0 && dy != 0 {
                    continue;
                }

                let index = IVec2 {
                    x: (node.index.x as i32 + dx),
                    y: (node.index.y as i32 + dy),
                };
                if let Some(index) =
                    self.get_or_insert_new(index, self.dest, tilemap, path_tiles_query)
                {
                    result.push(index);
                };
            }
        }

        dbg!(&self.to_explore);
        result
    }

    #[inline]
    pub fn is_explored(&self, index: UVec2) -> bool {
        self.explored.contains(&index)
    }

    #[inline]
    pub fn is_scheduled(&self, index: UVec2) -> bool {
        if let Some(Some(node)) = self
            .to_explore
            .get(self.index_to_path_node[&index].heap_index)
        {
            node.1 == index
        } else {
            false
        }
    }

    pub fn get(&self, index: UVec2) -> Option<&PathNode> {
        self.index_to_path_node.get(&index)
    }

    pub fn get_mut(&mut self, index: UVec2) -> Option<&mut PathNode> {
        self.index_to_path_node.get_mut(&index)
    }

    fn get_or_insert_new(
        &mut self,
        index: IVec2,
        dest: UVec2,
        tilemap: &Tilemap,
        path_tiles_query: &Query<&PathTile>,
    ) -> Option<UVec2> {
        println!("trying to get or insert new: {:?}", index);

        if tilemap.is_out_of_tilemap_ivec(index) {
            return None;
        }

        let index = index.as_uvec2();

        if self.is_explored(index) {
            return None;
        }

        let Some(tile_entity) = tilemap.get(index) else {
            return None;
        };

        let Ok(tile) = path_tiles_query.get(tile_entity) else {
            return None;
        };

        if !self.index_to_path_node.contains_key(&index) {
            self.index_to_path_node
                .insert(index, PathNode::new(index, 0, dest, 0, tile.cost));
        }

        Some(index)
    }

    pub fn schedule(&mut self, node: &UVec2) {
        if self.to_explore.len() == self.count + 1 {
            self.expand();
        }

        self.count += 1;
        let node = self.index_to_path_node.get_mut(node).unwrap();
        node.heap_index = self.count;
        self.to_explore[self.count] = Some((node.weight(self.weights), node.index));
        self.shift_up(self.count);
        println!(
            "scheduled: {:?} -> {:?} count = {}",
            self.to_explore[self.count], self.to_explore, self.count
        );
    }

    pub fn pop_closest(&mut self) -> Option<PathNode> {
        if self.count == 0 {
            return None;
        }

        println!("popping closest: {:?}", self.to_explore);
        let result = self.to_explore[1].unwrap().1;
        self.count -= 1;
        self.to_explore[1] = self.to_explore[self.count];
        self.shift_down(1);
        self.explored.insert(result);
        Some(self.index_to_path_node[&result])
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    fn expand(&mut self) {
        self.to_explore
            .extend_from_slice(&vec![None; self.depth * 2]);
        self.depth += 1;
    }

    fn shift_up(&mut self, index: usize) {
        if index == 1 {
            return;
        }

        println!("shifting up: {:?}", self.to_explore);
        let mut this = self.to_explore[index].unwrap();
        let mut parent = self.to_explore[index / 2].unwrap();

        while parent.0 > this.0 {
            let nodes = self
                .index_to_path_node
                .get_many_mut([&this.1, &parent.1])
                .unwrap();

            let tmp = nodes[0].heap_index;
            nodes[0].heap_index = nodes[1].heap_index;
            nodes[1].heap_index = tmp;

            let this_node = self.index_to_path_node.get(&this.1).unwrap();
            let parent_node = self.index_to_path_node.get(&parent.1).unwrap();
            println!(
                "shift up: {:?}({}) -> {:?}({})",
                this, this_node.heap_index, parent, parent_node.heap_index
            );

            self.to_explore
                .swap(parent_node.heap_index, this_node.heap_index);

            // Notice! Here because of the swap, the parent_node is actually this_node
            if parent_node.heap_index == 1 {
                break;
            } else {
                this = self.to_explore[parent_node.heap_index].unwrap();
                parent = self.to_explore[parent_node.heap_index / 2].unwrap();
            }
        }
    }

    fn shift_down(&mut self, index: usize) {
        println!("shifting down: {}", index);
        // dbg!(&self.to_explore);

        if self.to_explore[index].is_none() {
            return;
        }

        let mut this = self.to_explore[index].unwrap();
        let mut child = self.to_explore[index * 2].unwrap();

        while child.0 < this.0 {
            let nodes = self
                .index_to_path_node
                .get_many_mut([&this.1, &child.1])
                .unwrap();

            let tmp = nodes[0].heap_index;
            nodes[0].heap_index = nodes[1].heap_index;
            nodes[1].heap_index = tmp;

            let this_node = self.index_to_path_node.get(&this.1).unwrap();
            let child_node = self.index_to_path_node.get(&child.1).unwrap();

            self.to_explore
                .swap(child_node.heap_index, this_node.heap_index);

            // Notice! Here because of the swap, the child_node is actually this_node
            if child_node.heap_index * 2 >= self.count {
                break;
            } else {
                println!("shifting down: {}", child_node.heap_index);
                this = self.to_explore[child_node.heap_index].unwrap();
                child = {
                    let left = self.to_explore[child_node.heap_index * 2].unwrap();
                    let right = self.to_explore[child_node.heap_index * 2 + 1].unwrap();
                    if left.0 < right.0 {
                        left
                    } else {
                        right
                    }
                };
            }
        }
    }
}

pub fn pathfinding(
    commands: ParallelCommands,
    mut finders: Query<(Entity, &Pathfinder)>,
    tilemaps_query: Query<&Tilemap>,
    path_tiles_query: Query<&PathTile>,
) {
    finders
        .par_iter_mut()
        .for_each_mut(|(finder_entity, finder)| {
            #[cfg(feature = "debug")]
            println!("pathfinding start! {} -> {}", finder.origin, finder.dest);
            let tilemap = &tilemaps_query.get(finder.tilemap).unwrap();
            let weights = finder.custom_weight.unwrap_or((1, 1));

            // check if origin or dest doesn't exists
            if tilemap.is_out_of_tilemap_uvec(finder.origin)
                || tilemap.is_out_of_tilemap_uvec(finder.dest)
            {
                #[cfg(feature = "debug")]
                println!("out of tilemap");
                complete_pathfinding(&commands, finder_entity, None);
                return;
            };

            // initialize containers
            // only path_records stores the actual node data
            // which acts as a lookup table
            // the others only store the index
            let origin_node = PathNode::new(finder.origin, 0, finder.dest, 1, 0);
            let mut path_grid = PathGrid::new(finder, &origin_node);

            #[cfg(feature = "debug")]
            let mut i = 0;

            while !path_grid.is_empty() {
                #[cfg(feature = "debug")]
                {
                    i += 1;
                    println!("cycle {}", i);
                }

                let mut current = path_grid.pop_closest().unwrap();
                println!("current: {:?}", current);

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

                    #[cfg(feature = "debug")]
                    println!(
                        "pathfinding finished! after {} steps, length = {}",
                        i,
                        path.path.len()
                    );
                    complete_pathfinding(&commands, finder_entity, Some(path));
                    return;
                }

                let neighbours = {
                    if finder.allow_diagonal {
                        path_grid.neighbours(&current, tilemap, &path_tiles_query)
                    } else {
                        path_grid.neighbours(&current, tilemap, &path_tiles_query)
                    }
                };

                println!("neighbours: {:?}", neighbours);

                // explore neighbours
                for neighbour in neighbours {
                    let already_scheduled = path_grid.is_scheduled(neighbour);
                    let neighbour_node = path_grid.get_mut(neighbour).unwrap();

                    println!(
                        "exploring neighbour: {:?}, already_scheduled: {}",
                        neighbour_node, already_scheduled
                    );

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

            println!("stopped at {}", i);
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

mod test {
    use super::*;

    #[test]
    fn test_path_grid() {
        let finder = Pathfinder {
            origin: UVec2::new(0, 0),
            dest: UVec2::new(10, 10),
            allow_diagonal: false,
            tilemap: Entity::PLACEHOLDER,
            custom_weight: None,
        };
        let origin = PathNode::new(UVec2::new(0, 0), 0, finder.dest, 1, 0);
        let mut grid = PathGrid::new(&finder, &origin);
        // grid.neighbours(&origin, UVec2::new(15, 15));

        // grid.schedule(&UVec2::new(2, 2));
        // grid.schedule(&UVec2::new(4, 4));
        // grid.schedule(&UVec2::new(3, 3));
        // grid.schedule(&UVec2::new(1, 1));
        // grid.schedule(&UVec2::new(5, 5));

        // assert_eq!(grid.to_explore.len(), 64);
        // assert_eq!(grid.count, 31);
        // assert_eq!(grid.to_explore[1].unwrap().1, UVec2::new(0, 0));
        // assert_eq!(grid.to_explore[2].unwrap().1, UVec2::new(1, 1));
        // assert_eq!(grid.to_explore[3].unwrap().1, UVec2::new(2, 2));
        // assert_eq!(grid.to_explore[4].unwrap().1, UVec2::new(3, 3));
        // assert_eq!(grid.to_explore[5].unwrap().1, UVec2::new(4, 4));
        // assert_eq!(grid.to_explore[6].unwrap().1, UVec2::new(5, 5));
    }
}
