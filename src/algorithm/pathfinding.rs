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
    pub max_step: Option<u32>,
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
            to_explore: Vec::from([None, Some((root.weight(weights), root.index)), None, None]),
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
                    self.get_or_register_new(index, self.dest, tilemap, path_tiles_query)
                {
                    result.push(index);
                };
            }
        }
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

    fn get_or_register_new(
        &mut self,
        index: IVec2,
        dest: UVec2,
        tilemap: &Tilemap,
        path_tiles_query: &Query<&PathTile>,
    ) -> Option<UVec2> {
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
        println!("scheduled: {:?}", self.to_explore[self.count]);
        self.shift_up(self.count);
        self.print_info();
        self.check();
    }

    pub fn pop_closest(&mut self) -> Option<PathNode> {
        if self.count == 0 {
            return None;
        }

        let result = self.to_explore[1].unwrap();
        self.to_explore[1] = self.to_explore[self.count];
        self.index_to_path_node
            .get_mut(&self.to_explore[1].unwrap().1)
            .unwrap()
            .heap_index = 1;
        self.to_explore[self.count] = None;
        self.count -= 1;
        println!("popped: {:?}", result);
        self.print_info();
        self.shift_down(1);
        self.explored.insert(result.1);
        let res = Some(self.index_to_path_node[&result.1]);
        self.print_info();
        self.check();
        res
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
        let Some(mut this) = self.to_explore[index] else {
            return;
        };
        let Some(mut parent) = self.to_explore[index / 2] else {
            return;
        };
        println!("try shift up: {:?} -> {:?}", this, parent);

        while parent.0 > this.0 {
            let (swapped_this, _) = self.swap_node(this.1, parent.1);
            self.print_info();

            if swapped_this == 1 {
                break;
            } else {
                this = self.to_explore[swapped_this].unwrap();
                parent = self.to_explore[swapped_this / 2].unwrap();
                println!("try shift up: {:?} -> {:?}", this, parent);
            }
        }
    }

    fn shift_down(&mut self, index: usize) {
        if index * 2 >= self.count {
            return;
        };
        let Some(mut this) = self.to_explore[index] else {
            return;
        };
        let mut child = {
            let left = self.to_explore[index * 2].unwrap();
            let right = self.to_explore[index * 2 + 1].unwrap();
            println!("selecting: {:?} <-> {:?}", left, right);
            if left.0 <= right.0 {
                left
            } else {
                right
            }
        };

        while child.0 < this.0 {
            let (swapped_this, _) = self.swap_node(this.1, child.1);
            self.print_info();

            if swapped_this * 2 > self.count {
                break;
            } else {
                this = self.to_explore[swapped_this].unwrap();
                child = {
                    let left = self.to_explore[swapped_this * 2].unwrap();
                    if let Some(right) = self.to_explore[swapped_this * 2 + 1]{
                        println!("selecting: {:?} <-> {:?}", left, right);
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
    fn swap_node(&mut self, lhs_index: UVec2, rhs_index: UVec2) -> (usize, usize) {
        println!(
            "----------\nswapping: {} <-> {}----------",
            lhs_index, rhs_index,
        );
        self.print_info();

        let nodes = self
            .index_to_path_node
            .get_many_mut([&lhs_index, &rhs_index])
            .unwrap();

        let tmp = nodes[0].heap_index;
        nodes[0].heap_index = nodes[1].heap_index;
        nodes[1].heap_index = tmp;

        self.to_explore
            .swap(nodes[0].heap_index, nodes[1].heap_index);

        println!(
            "swapped: {}({}) <-> {}({})",
            nodes[0].index,
            self.to_explore[nodes[0].heap_index].unwrap().0,
            nodes[1].index,
            self.to_explore[nodes[1].heap_index].unwrap().0
        );

        (nodes[0].heap_index, nodes[1].heap_index)
    }

    fn print_info(&self) {
        println!(
            "----------\nindex_to_path: {:?}\nto_explore: {:?}\n----------",
            self.index_to_path_node
                .iter()
                .filter(|e| e.1.heap_index != 1)
                .map(|e| (e.0, e.1.heap_index))
                .collect::<Vec<_>>(),
            self.to_explore
        );
    }

    fn check(&self) {
        for i in 2..=self.count {
            let Some(this) = self.to_explore[i] else {
                continue;
            };
            let parent = self.to_explore[i / 2].unwrap();
            assert!(
                parent.0 <= this.0,
                "check failed! {:?} > {:?}",
                parent,
                this
            );
        }

        for i in 1..self.count {
            let Some(this) = self.to_explore[i] else {
                continue;
            };

            assert_eq!(self.index_to_path_node[&this.1].heap_index, i)
        }

        println!("check passed √");
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

            let mut step = 0;

            while !path_grid.is_empty() {
                step += 1;
                if step > finder.max_step.unwrap_or(u32::MAX) {
                    break;
                }

                println!("cycle {}", step);

                let mut current = path_grid.pop_closest().unwrap();
                println!("current: {:?}", current);

                if current.index == finder.dest {
                    let mut path = Path {
                        path: vec![],
                        current_step: 0,
                        target_map: finder.tilemap,
                    };
                    println!("path found!");
                    while current.index != finder.origin {
                        println!("tracing back: {:?} -> {:?}", current.index, current.parent);
                        path.path.push(current.index);
                        current = *path_grid.get(current.parent.unwrap()).unwrap();
                    }

                    #[cfg(feature = "debug")]
                    println!(
                        "pathfinding finished! after {} steps, length = {}",
                        step,
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

                println!("to_explore: {:?}", path_grid.to_explore);
                path_grid.check();
                println!("==============================");
            }

            println!("stopped at {}", step);
            complete_pathfinding(&commands, finder_entity, None);
        });
}

pub fn complete_pathfinding(commands: &ParallelCommands, finder: Entity, path: Option<Path>) {
    #[cfg(feature = "debug")]
    if path.is_none() {
        println!("path not found");
        panic!();
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
        // let finder = Pathfinder {
        //     origin: UVec2::new(0, 0),
        //     dest: UVec2::new(10, 10),
        //     allow_diagonal: false,
        //     tilemap: Entity::PLACEHOLDER,
        //     custom_weight: None,
        // };
        // let origin = PathNode::new(UVec2::new(0, 0), 0, finder.dest, 1, 0);
        // let mut grid = PathGrid::new(&finder, &origin);
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
