/// Direction order: up, right, left, down
use std::fs::read_to_string;

use bevy::{
    ecs::entity::Entity,
    math::IVec2,
    prelude::{Commands, Component, ParallelCommands, Query, UVec2},
    utils::HashSet,
};
use indexmap::IndexSet;
use rand::{distributions::Uniform, rngs::StdRng, Rng, SeedableRng};
use ron::de::from_bytes;

use crate::{
    math::FillArea,
    tilemap::{TileBuilder, Tilemap},
};

#[derive(Default, Clone, Copy)]
pub enum WfcMode {
    #[default]
    NonWeighted,
}

#[derive(Component)]
pub struct WaveFunctionCollapser {
    pub rule: Vec<[IndexSet<u16>; 4]>,
    pub mode: WfcMode,
    pub seed: Option<u64>,
    pub area: FillArea,
    pub step_interval: Option<f32>,
}

impl WaveFunctionCollapser {
    /// The order of the directions should be: up, right, left, down.
    pub fn from_config(
        config_path: String,
        mode: WfcMode,
        area: FillArea,
        step_interval: Option<f32>,
        seed: Option<u64>,
    ) -> Self {
        let rule_vec: Vec<[Vec<u16>; 4]> =
            from_bytes(read_to_string(config_path).unwrap().as_bytes()).unwrap();

        let mut rule: Vec<[IndexSet<u16>; 4]> = Vec::with_capacity(rule_vec.len());
        for tex_idx in 0..rule_vec.len() {
            let mut tex_rule: [IndexSet<u16>; 4] = Default::default();
            for dir in 0..4 {
                for idx in rule_vec[tex_idx][dir].iter() {
                    tex_rule[dir].insert(*idx);
                }
            }
            rule.push(tex_rule);
        }

        Self {
            rule,
            mode,
            area,
            step_interval,
            seed,
        }
    }
}

#[derive(Debug, Clone)]
struct WfcTile {
    pub index: UVec2,
    pub collapsed: bool,
    pub texture_index: Option<u16>,
    pub heap_index: usize,
    pub psbs: IndexSet<u16>,
}

struct WfcHistory {
    grid: Vec<WfcTile>,
    heap: Vec<(u16, UVec2)>,
    remaining: usize,
}

struct WfcGrid {
    mode: WfcMode,
    history: Vec<WfcHistory>,
    size: UVec2,
    grid: Vec<WfcTile>,
    max_psbs: u16,
    rng: StdRng,
    rule: Vec<[IndexSet<u16>; 4]>,
    heap: Vec<(u16, UVec2)>,
    remaining: usize,
    retrace_strength: u32,
}

impl WfcGrid {
    pub fn from_collapser(collapser: &WaveFunctionCollapser) -> Self {
        let mut grid = Vec::with_capacity(collapser.area.size());
        let mut heap = Vec::with_capacity(collapser.area.size() + 1);
        let max_psbs = collapser.rule.len() as u16;
        // a placeholder
        heap.push((0, UVec2::new(0, 0)));
        let mut heap_index = 1;

        for y in 0..collapser.area.extent.y {
            for x in 0..collapser.area.extent.x {
                grid.push(WfcTile {
                    heap_index,
                    index: UVec2 { x, y },
                    texture_index: None,
                    collapsed: false,
                    psbs: (0..max_psbs).collect(),
                });

                heap.push((collapser.rule.len() as u16, UVec2 { x, y }));
                heap_index += 1;
            }
        }

        WfcGrid {
            grid,
            size: collapser.area.extent,
            history: vec![],
            mode: collapser.mode,
            max_psbs,
            rule: collapser.rule.clone(),
            rng: match collapser.seed {
                Some(seed) => StdRng::seed_from_u64(seed),
                None => StdRng::from_entropy(),
            },
            heap,
            remaining: collapser.area.size(),
            retrace_strength: 1,
        }
    }

    pub fn get_tile(&self, index: UVec2) -> Option<&WfcTile> {
        self.grid.get((index.y * self.size.x + index.x) as usize)
    }

    pub fn get_tile_mut(&mut self, index: UVec2) -> Option<&mut WfcTile> {
        self.grid
            .get_mut((index.y * self.size.x + index.x) as usize)
    }

    pub fn pick_random(&self) -> UVec2 {
        let mut rng = self.rng.clone();
        let x = rng.gen_range(0..self.size.x);
        let y = rng.gen_range(0..self.size.y);
        UVec2::new(x, y)
    }

    pub fn is_out_of_grid(&self, index: UVec2) -> bool {
        index.x >= self.size.x || index.y >= self.size.y
    }

    pub fn pop_min(&mut self) -> WfcTile {
        let hist = WfcHistory {
            grid: self.grid.clone(),
            remaining: self.remaining,
            heap: self.heap.clone(),
        };
        self.history.push(hist);

        let min_tile = self.get_tile(self.heap[1].1).unwrap().clone();
        self.remaining -= 1;

        #[cfg(feature = "debug_verbose")]
        println!("popped: {}, remaining={}", min_tile.index, self.remaining);
        if self.remaining > 0 {
            let max = self.heap.pop().unwrap();
            self.heap[1] = max;
            self.get_tile_mut(self.heap[1].1).unwrap().heap_index = 1;
            self.shift_down(1);
        }
        min_tile
    }

    pub fn collapse(&mut self, index: UVec2) {
        let Some(tile) = self.get_tile(index) else {
            return;
        };
        if tile.collapsed {
            return;
        }

        #[cfg(feature = "debug_verbose")]
        {
            println!("collasping: {:?}", index);
            self.print_grid();
        }

        let index = tile.index;
        let entropy = tile.psbs.len() as usize;
        if entropy == 1 {
            self.retrace_strength *= 2;
        } else {
            self.retrace_strength = 1;
        }
        let rd = match self.mode {
            WfcMode::NonWeighted => self.rng.sample(Uniform::new(0, entropy)),
        };

        let tile = self.get_tile_mut(index).unwrap();
        tile.texture_index = Some(tile.psbs[rd]);
        tile.collapsed = true;

        tile.psbs = IndexSet::from([tile.psbs[rd]]);

        self.spread_constraint(index);
    }

    pub fn spread_constraint(&mut self, center: UVec2) {
        let mut queue: Vec<UVec2> = vec![center];
        let mut spreaded = HashSet::default();

        while !queue.is_empty() {
            let cur_ctr = queue.pop().unwrap();
            #[cfg(feature = "debug_verbose")]
            println!("constraining: {}'s neighbour", cur_ctr);
            spreaded.insert(cur_ctr);
            let cur_tile = self.get_tile(cur_ctr).unwrap().clone();

            let neighbours = self.neighbours(cur_ctr);
            let mut psbs_cache = vec![IndexSet::default(); 4];

            // constrain
            for dir in 0..neighbours.len() {
                let neighbour_tile = self.get_tile(neighbours[dir]).unwrap();
                if neighbour_tile.collapsed || spreaded.contains(&neighbours[dir]) {
                    #[cfg(feature = "debug_verbose")]
                    println!("skipping neighbour: {:?}", neighbours[dir]);
                    continue;
                }
                #[cfg(feature = "debug_verbose")]
                println!("constraining: {:?}", neighbours[dir]);

                for psb in cur_tile.psbs.iter() {
                    psbs_cache[dir] = psbs_cache[dir]
                        .union(&self.rule[*psb as usize][dir])
                        .map(|e| *e)
                        .collect::<IndexSet<u16>>();
                }
                psbs_cache[dir] = psbs_cache[dir]
                    .intersection(&neighbour_tile.psbs)
                    .map(|e| *e)
                    .collect::<IndexSet<u16>>();
                #[cfg(feature = "debug_verbose")]
                println!(
                    "{}'s psbs: {:?}, dir={})",
                    neighbours[dir], psbs_cache[dir], dir
                );
                if psbs_cache[dir].len() == 0 {
                    #[cfg(feature = "debug_verbose")]
                    println!("start retrace because of: {}", neighbours[dir]);
                    self.retrace();
                    return;
                }
                spreaded.insert(cur_ctr);
                queue.push(neighbours[dir]);
            }

            for dir in 0..neighbours.len() {
                let tile = self.get_tile_mut(neighbours[dir]).unwrap();
                if !tile.collapsed && !spreaded.contains(&tile.index) {
                    tile.psbs = psbs_cache[dir].clone();
                    self.update_entropy(neighbours[dir]);
                }
            }

            // #[cfg(feature = "debug")]
            // self.print_grid();
        }
    }

    pub fn retrace(&mut self) {
        #[cfg(feature = "debug")]
        println!("retrace with strength: {}", self.retrace_strength);
        for _ in 0..self.retrace_strength - 1 {
            self.history.pop();
        }
        let hist = self.history.pop().unwrap();
        self.grid = hist.grid;
        self.remaining = hist.remaining;
        self.heap = hist.heap;
        #[cfg(feature = "debug_verbose")]
        {
            self.validate();
            println!("retrace success");
        }
    }

    pub fn apply_map(&self, commands: &mut Commands, tilemap: &mut Tilemap) {
        #[cfg(feature = "debug_verbose")]
        {
            println!("map collapsed!");
            self.print_grid();
        }
        for tile in self.grid.iter() {
            let index = tile.index;
            let texture_index = tile.texture_index.unwrap() as u32;
            tilemap.set(commands, TileBuilder::new(index, texture_index));
        }
    }

    pub fn neighbours(&mut self, index: UVec2) -> Vec<UVec2> {
        let index = index.as_ivec2();
        vec![
            IVec2::new(index.x, index.y + 1),
            IVec2::new(index.x + 1, index.y),
            IVec2::new(index.x - 1, index.y),
            IVec2::new(index.x, index.y - 1),
        ]
        .iter()
        .filter(|p| p.x >= 0 && p.y >= 0 && p.x < self.size.x as i32 && p.y < self.size.y as i32)
        .map(|p| p.as_uvec2())
        .collect::<Vec<_>>()
    }

    fn update_entropy(&mut self, index: UVec2) {
        if self.is_out_of_grid(index) {
            return;
        }

        let tile = self.get_tile_mut(index).unwrap();
        let heap_index = tile.heap_index;
        self.heap[heap_index].0 = tile.psbs.len() as u16;
        self.shift_up(heap_index);
    }

    fn shift_up(&mut self, index: usize) {
        let Some(mut this) = self.heap.get(index) else {
            return;
        };
        let Some(mut parent) = self.heap.get(index / 2) else {
            return;
        };

        while parent.0 > this.0 {
            let (swapped_this, _) = self.swap_node(this.1, parent.1);

            if swapped_this == 1 {
                break;
            } else {
                this = self.heap.get(swapped_this).unwrap();
                parent = self.heap.get(swapped_this / 2).unwrap();
            }
        }
    }

    fn shift_down(&mut self, index: usize) {
        if index * 2 > self.heap.len() - 1 {
            return;
        };
        let Some(mut this) = self.heap.get(index) else {
            return;
        };
        let mut child = {
            let left = self.heap.get(index * 2).unwrap();
            if let Some(right) = self.heap.get(index * 2 + 1) {
                if left.0 < right.0 {
                    left
                } else {
                    right
                }
            } else {
                left
            }
        };

        while child.0 < this.0 {
            let (swapped_this, _) = self.swap_node(this.1, child.1);

            if swapped_this * 2 > self.heap.len() - 1 {
                break;
            } else {
                this = self.heap.get(swapped_this).unwrap();
                child = {
                    let left = self.heap.get(swapped_this * 2).unwrap();
                    if let Some(right) = self.heap.get(swapped_this * 2 + 1) {
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
        let lhs_heap_index = self.get_tile(lhs_index).unwrap().heap_index;
        let rhs_heap_index = self.get_tile(rhs_index).unwrap().heap_index;

        self.heap.swap(lhs_heap_index, rhs_heap_index);

        self.get_tile_mut(lhs_index).unwrap().heap_index = rhs_heap_index;
        self.get_tile_mut(rhs_index).unwrap().heap_index = lhs_heap_index;

        (rhs_heap_index, lhs_heap_index)
    }

    fn print_grid(&self) {
        let mut result = "-------------------\n".to_string();
        let mut counter = 0;

        for t in self.grid.iter() {
            result.push_str(&format!(
                "{:?}{}({:?})\t",
                t.texture_index,
                t.index,
                self.get_tile(t.index).unwrap().psbs
            ));
            counter += 1;
            if counter == self.size.x {
                result.push_str("\n");
                counter = 0;
            }
        }

        result.push_str("-------------------");
        println!("{}", result);
    }

    fn validate(&self) {
        crate::debug::validate_heap(&self.heap, true);
        for i in 1..self.heap.len() {
            assert_eq!(
                self.get_tile(self.heap[i].1).unwrap().heap_index,
                i,
                "heap index not match at: {}",
                self.heap[i].1
            );
        }
    }
}

pub fn wave_function_collapse(
    commands: ParallelCommands,
    mut collapser_query: Query<(Entity, &mut Tilemap, &WaveFunctionCollapser)>,
) {
    collapser_query
        .par_iter_mut()
        .for_each_mut(|(entity, mut tilemap, collapser)| {
            #[cfg(feature = "debug")]
            let start = std::time::SystemTime::now();
            let mut wfc_grid = WfcGrid::from_collapser(&collapser);
            wfc_grid.collapse(wfc_grid.pick_random());

            while wfc_grid.remaining > 0 {
                #[cfg(feature = "debug_verbose")]
                println!("=============================");
                let min_tile = wfc_grid.pop_min();
                wfc_grid.collapse(min_tile.index);
                #[cfg(feature = "debug_verbose")]
                {
                    println!("cycle complete, start validating");
                    wfc_grid.validate();
                    println!("=============================");
                }
                #[cfg(feature = "debug")]
                println!("remaining: {}", wfc_grid.remaining);
            }

            #[cfg(feature = "debug")]
            println!("wfc complete in: {:?}", start.elapsed().unwrap());

            commands.command_scope(|mut c| {
                wfc_grid.apply_map(&mut c, &mut tilemap);
                c.entity(entity).remove::<WaveFunctionCollapser>();
            })
        });
}
