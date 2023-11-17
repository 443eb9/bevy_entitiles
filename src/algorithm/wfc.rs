/// Direction order: up, right, left, down
use std::fs::read_to_string;

use bevy::{
    ecs::entity::Entity,
    math::IVec2,
    prelude::{Commands, Component, ParallelCommands, Query, UVec2},
    utils::HashSet,
};
use indexmap::IndexSet;
use rand::{
    distributions::{Uniform, WeightedIndex},
    rngs::StdRng,
    Rng, SeedableRng,
};
use ron::de::from_bytes;

use crate::{
    math::FillArea,
    tilemap::{TileBuilder, Tilemap},
};

#[derive(Default, Clone, PartialEq, Eq)]
pub enum WfcMode {
    #[default]
    /// Randomly pick one from the possibilities.
    NonWeighted,
    /// Pick one from the possibilities according to the weights.
    Weighted(Vec<u8>),
    /// This mode requires a function UVec2 -> u16,
    /// which you can see in the `WaveFunctionCollapser::from_config` method
    ///
    /// You can use this to generate a map according to a noise function etc.
    CustomSampler,
}

/// # Warning!
/// This feature is still in preview. It may fail to generate a map and stuck in an infinite loop.
#[derive(Component)]
pub struct WfcRunner {
    rule: Vec<[IndexSet<u16>; 4]>,
    mode: WfcMode,
    sampler: Option<Box<dyn Fn(&IndexSet<u16>, UVec2) -> u16 + Send + Sync>>,
    seed: Option<u64>,
    area: FillArea,
    step_interval: Option<f32>,
    max_retrace_factor: u32,
    max_retrace_time: u32,
    fallback: Option<Box<dyn Fn(Entity) + Send + Sync>>,
}

impl WfcRunner {
    /// The order of the directions in config should be: up, right, left, down.
    ///
    /// Don't input `weights_path` and `custom_sampler` at the same time.
    ///
    /// Please make sure the length of the `weights` is the same as the length of the `rule`.
    ///
    /// **Currently not implemented:**
    /// `step_interval` is the interval between each step in miliseconds.
    /// If you want to visualize the process, you can set this to some value.
    ///
    /// `seed` is the seed of the random number generator. Leave it `None` if you want to use a random seed.
    pub fn from_config(
        config_path: String,
        weights_path: Option<String>,
        custom_sampler: Option<Box<dyn Fn(&IndexSet<u16>, UVec2) -> u16 + Send + Sync>>,
        area: FillArea,
        step_interval: Option<f32>,
        seed: Option<u64>,
    ) -> Self {
        let rule_vec: Vec<[Vec<u16>; 4]> =
            from_bytes(read_to_string(config_path).unwrap().as_bytes()).unwrap();

        let mut mode = WfcMode::NonWeighted;
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

        if weights_path.is_some() && custom_sampler.is_some() {
            panic!("You can only use one of the weights and custom_sampler at the same time!");
        }

        if let Some(weights_path) = weights_path {
            let weights_vec: Vec<u8> =
                from_bytes(read_to_string(weights_path).unwrap().as_bytes()).unwrap();
            assert_eq!(
                weights_vec.len(),
                rule.len(),
                "weights length not match! weights: {}, rules: {}",
                weights_vec.len(),
                rule.len()
            );
            mode = WfcMode::Weighted(weights_vec);
        }

        let mut sampler = None;
        if let Some(_) = custom_sampler {
            mode = WfcMode::CustomSampler;
            sampler = custom_sampler;
        }

        Self {
            rule,
            mode,
            sampler,
            area,
            step_interval,
            seed,
            max_retrace_factor: 8,
            max_retrace_time: 200,
            fallback: None,
        }
    }

    /// Set the retrace settings. This will affect the **probability of success**.
    /// The higher those parameters are, the higher the probability of success is.
    /// But it will also dramatically increase the time cost.
    ///
    /// Default: `max_retrace_factor` = 8, `max_retrace_time` = 200
    pub fn with_retrace_settings(mut self, max_retrace_factor: u32, max_retrace_time: u32) -> Self {
        self.max_retrace_factor = max_retrace_factor;
        self.max_retrace_time = max_retrace_time;
        self
    }

    /// Set the fallback function. This function will be called when the algorithm failed to generate a map.
    pub fn with_fallback(mut self, fallback: Box<dyn Fn(Entity) + Send + Sync>) -> Self {
        self.fallback = Some(fallback);
        self
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
    rng: StdRng,
    rule: Vec<[IndexSet<u16>; 4]>,
    sampler: Option<Box<dyn Fn(&IndexSet<u16>, UVec2) -> u16 + Send + Sync>>,
    heap: Vec<(u16, UVec2)>,
    remaining: usize,
    retrace_strength: u32,
    max_retrace_factor: u32,
    max_retrace_time: u32,
    retraced_time: u32,
    fallback: Option<Box<dyn Fn(Entity) + Send + Sync>>,
}

impl WfcGrid {
    pub fn from_runner(runner: &mut WfcRunner) -> Self {
        let mut grid = Vec::with_capacity(runner.area.size());
        let mut heap = Vec::with_capacity(runner.area.size() + 1);
        let max_psbs = runner.rule.len() as u16;
        // a placeholder
        heap.push((0, UVec2::new(0, 0)));
        let mut heap_index = 1;

        for y in 0..runner.area.extent.y {
            for x in 0..runner.area.extent.x {
                grid.push(WfcTile {
                    heap_index,
                    index: UVec2 { x, y },
                    texture_index: None,
                    collapsed: false,
                    psbs: (0..max_psbs).collect(),
                });

                heap.push((runner.rule.len() as u16, UVec2 { x, y }));
                heap_index += 1;
            }
        }

        WfcGrid {
            grid,
            size: runner.area.extent,
            history: vec![],
            mode: runner.mode.clone(),
            rule: runner.rule.clone(),
            rng: match runner.seed {
                Some(seed) => StdRng::seed_from_u64(seed),
                None => StdRng::from_entropy(),
            },
            heap,
            remaining: runner.area.size(),
            retrace_strength: 1,
            max_retrace_factor: runner.max_retrace_factor,
            max_retrace_time: runner.max_retrace_time,
            retraced_time: 0,
            sampler: runner.sampler.take(),
            fallback: runner.fallback.take(),
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

        let psb = match &self.mode {
            WfcMode::NonWeighted => self.rng.sample(Uniform::new(0, entropy)),
            WfcMode::Weighted(w) => {
                let weights = tile.psbs.iter().map(|p| w[*p as usize]).collect::<Vec<_>>();
                self.rng.sample(WeightedIndex::new(weights).unwrap())
            }
            WfcMode::CustomSampler => self.sampler.as_ref().unwrap()(&tile.psbs, index) as usize,
        };

        self.retrace_strength *= self.rng.sample(Uniform::new(2, self.max_retrace_factor));

        let mode = self.mode.clone();
        let tile = self.get_tile_mut(index).unwrap();

        // because of the existing of ownership,
        // we can't map the indices to possibilities when mathing.
        if mode == WfcMode::CustomSampler {
            tile.texture_index = Some(psb as u16);
            tile.psbs = IndexSet::from([psb as u16]);
        } else {
            tile.texture_index = Some(tile.psbs[psb]);
            tile.psbs = IndexSet::from([tile.psbs[psb]]);
        }

        tile.collapsed = true;

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

        self.retrace_strength = 1;
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
        self.retraced_time += 1;
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

    #[cfg(feature = "debug_verbose")]
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

    #[cfg(feature = "debug_verbose")]
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
    mut collapser_query: Query<(Entity, &mut Tilemap, &mut WfcRunner)>,
) {
    collapser_query
        .par_iter_mut()
        .for_each(|(entity, mut tilemap, mut collapser)| {
            #[cfg(feature = "debug")]
            let start = std::time::SystemTime::now();
            let mut wfc_grid = WfcGrid::from_runner(&mut collapser);
            wfc_grid.collapse(wfc_grid.pick_random());

            while wfc_grid.remaining > 0 && wfc_grid.retraced_time < wfc_grid.max_retrace_time {
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
                if wfc_grid.retraced_time < wfc_grid.max_retrace_time {
                    wfc_grid.apply_map(&mut c, &mut tilemap);
                } else if let Some(fallback) = wfc_grid.fallback {
                    fallback(entity);
                }
                c.entity(entity).remove::<WfcRunner>();
            })
        });
}
