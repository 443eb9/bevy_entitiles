/// Direction order: up, right, left, down
use std::fs::read_to_string;

use bevy::{
    ecs::{entity::Entity, query::Without},
    math::IVec2,
    prelude::{Commands, Component, ParallelCommands, Query, UVec2},
    utils::HashSet,
};
use rand::{
    distributions::{Uniform, WeightedIndex},
    rngs::StdRng,
    Rng, SeedableRng,
};
use ron::de::from_bytes;

use crate::{
    math::FillArea,
    tilemap::{map::Tilemap, tile::TileBuilder},
};

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub enum WfcMode {
    #[default]
    /// Randomly pick one from the possibilities.
    NonWeighted,
    /// Pick one from the possibilities according to the weights.
    Weighted(Vec<u8>),
    /// This mode requires a function WfcTile -> u8,
    /// which you can see in the `WaveFunctionCollapser::from_config` method
    ///
    /// You can use this to generate a map according to a noise function etc.
    CustomSampler,
}

/// # Warning!
/// This feature is still in preview. It may fail to generate a map and stuck in an infinite loop.
#[derive(Component)]
pub struct WfcRunner {
    rule: Vec<[u128; 4]>,
    mode: WfcMode,
    sampler: Option<Box<dyn Fn(&WfcTile, &mut StdRng) -> u8 + Send + Sync>>,
    seed: Option<u64>,
    area: FillArea,
    max_retrace_factor: u32,
    max_retrace_count: u32,
    max_history: usize,
    fallback: Option<Box<dyn Fn(&mut Commands, Entity, &Tilemap, &WfcRunner) + Send + Sync>>,
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
    pub fn from_config(rule_path: String, area: FillArea, seed: Option<u64>) -> Self {
        let rule_vec: Vec<[Vec<u16>; 4]> =
            from_bytes(read_to_string(rule_path).unwrap().as_bytes()).unwrap();

        assert!(
            rule_vec.len() <= 128,
            "We only support 128 textures for now"
        );

        let mut rule_set: Vec<[Vec<u16>; 4]> = Vec::with_capacity(rule_vec.len());
        for tex_idx in 0..rule_vec.len() {
            let mut tex_rule: [Vec<u16>; 4] = Default::default();
            for dir in 0..4 {
                for idx in rule_vec[tex_idx][dir].iter() {
                    tex_rule[dir].push(*idx);
                }
            }
            rule_set.push(tex_rule);
        }

        let mut rule: Vec<[u128; 4]> = Vec::with_capacity(rule_set.len());
        for tex_idx in 0..rule_set.len() {
            let mut tex_rule: [u128; 4] = Default::default();
            for dir in 0..4 {
                for idx in rule_set[tex_idx][dir].iter() {
                    tex_rule[dir] |= 1 << idx;
                }
            }
            rule.push(tex_rule);
        }

        let size = area.size();

        Self {
            rule,
            mode: WfcMode::NonWeighted,
            sampler: None,
            area,
            seed,
            max_retrace_factor: size.ilog10().clamp(2, 16),
            max_retrace_count: size.ilog10().clamp(2, 16) * 100,
            max_history: (size.ilog10().clamp(1, 8) * 20) as usize,
            fallback: None,
        }
    }

    /// Set the weights of the tiles.
    /// The length of the weights should be the same as the length of the rule.
    pub fn with_weights(mut self, weights_path: String) -> Self {
        assert_eq!(
            self.mode,
            WfcMode::NonWeighted,
            "You can only use one sampler or one weights vector"
        );
        let weights_vec: Vec<u8> =
            from_bytes(read_to_string(weights_path).unwrap().as_bytes()).unwrap();
        assert_eq!(
            weights_vec.len(),
            self.rule.len(),
            "weights length not match! weights: {}, rules: {}",
            weights_vec.len(),
            self.rule.len()
        );
        self.mode = WfcMode::Weighted(weights_vec);
        self
    }

    /// Set the custom sampler function.
    /// The function should accept `WfcTile`,`StdRng` and return a `u8` as the texture index.
    pub fn with_custom_sampler(
        mut self,
        custom_sampler: Box<dyn Fn(&WfcTile, &mut StdRng) -> u8 + Send + Sync>,
    ) -> Self {
        assert_eq!(
            self.mode,
            WfcMode::NonWeighted,
            "You can only use one sampler or one weights vector"
        );
        self.mode = WfcMode::CustomSampler;
        self.sampler = Some(custom_sampler);
        self
    }

    /// Set the retrace settings. This will affect the **probability of success**.
    /// The higher those parameters are, the higher the probability of success is.
    /// But it will also dramatically increase the time cost.
    ///
    /// Default:
    /// `max_retrace_factor` = `size.ilog10().clamp(2, 16)`,
    /// `max_retrace_time` = `size.ilog10().clamp(2, 16) * 100`
    pub fn with_retrace_settings(
        mut self,
        max_retrace_factor: Option<u32>,
        max_retrace_count: Option<u32>,
    ) -> Self {
        if let Some(factor) = max_retrace_factor {
            assert!(factor <= 16, "max_retrace_factor should be <= 16");
            self.max_retrace_factor = factor;
        }
        if let Some(count) = max_retrace_count {
            self.max_retrace_count = count;
        }
        self
    }

    /// Set the history settings.
    /// The algorithm will retrace using the history when a tile has no possibilities.
    /// Lower `max_history` can save memory. But it will be more likely to fail.
    ///
    /// Default: `max_history` = `size.ilog10().clamp(1, 8) * 20`
    pub fn with_history_settings(mut self, max_history: usize) -> Self {
        self.max_history = max_history;
        self
    }

    /// Set the fallback function.
    /// This function will be called when the algorithm failed to generate a map.
    ///
    /// The Entity in the parameter is the entity that the `WfcRunner` is attached to.
    pub fn with_fallback(
        mut self,
        fallback: Box<dyn Fn(&mut Commands, Entity, &Tilemap, &WfcRunner) + Send + Sync>,
    ) -> Self {
        self.fallback = Some(fallback);
        self
    }
}

/// This will sharply increase the time cost.
/// Use it only when you **REALLY** want to visualize the process.
#[derive(Component)]
pub struct AsyncWfcRunner;

#[derive(Debug, Clone, Copy)]
pub struct WfcTile {
    pub index: UVec2,
    pub collapsed: bool,
    pub texture_index: Option<u8>,
    pub heap_index: usize,
    pub psbs: u128,
}

impl WfcTile {
    pub fn get_psbs_vec(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(self.psbs.count_ones() as usize);
        for i in 0..128 {
            if self.psbs & (1 << i) != 0 {
                result.push(i as u8);
            }
        }
        result
    }
}

#[derive(Clone)]
struct WfcHistory {
    grid: Vec<WfcTile>,
    heap: Vec<(u8, UVec2)>,
    remaining: usize,
}

#[derive(Component)]
pub struct WfcGrid {
    mode: WfcMode,
    history: Vec<WfcHistory>,
    size: UVec2,
    grid: Vec<WfcTile>,
    rng: StdRng,
    rule: Vec<[u128; 4]>,
    sampler: Option<Box<dyn Fn(&WfcTile, &mut StdRng) -> u8 + Send + Sync>>,
    heap: Vec<(u8, UVec2)>,
    remaining: usize,
    retrace_strength: u32,
    max_retrace_factor: u32,
    max_retrace_time: u32,
    retraced_time: u32,
    fallback: Option<Box<dyn Fn(&mut Commands, Entity, &Tilemap, &WfcRunner) + Send + Sync>>,
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
                    psbs: (!0) >> (128 - max_psbs),
                });

                heap.push((runner.rule.len() as u8, UVec2 { x, y }));
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
            max_retrace_time: runner.max_retrace_count,
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
        let entropy = tile.psbs.count_ones() as usize;

        let psb = match &self.mode {
            WfcMode::NonWeighted => {
                let psb_vec = tile.get_psbs_vec();
                psb_vec[self.rng.sample(Uniform::new(0, entropy))]
            }
            WfcMode::Weighted(w) => {
                let psb_vec = tile.get_psbs_vec();
                let weights = psb_vec.iter().map(|p| w[*p as usize]).collect::<Vec<_>>();
                psb_vec[self.rng.sample(WeightedIndex::new(weights).unwrap())]
            }
            WfcMode::CustomSampler => {
                let mut rng = self.rng.clone();
                let res = self.sampler.as_ref().unwrap()(&tile, &mut rng) as u8;
                self.rng = rng;
                res
            }
        };

        self.retrace_strength *= self.rng.sample(Uniform::new(1, self.max_retrace_factor));

        let tile = self.get_tile_mut(index).unwrap();
        tile.texture_index = Some(psb);
        tile.psbs = 1 << psb;
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
            let mut psbs_cache = vec![0; 4];

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

                for i in 0..self.rule.len() {
                    if cur_tile.psbs & (1 << i) != 0 {
                        psbs_cache[dir] |= self.rule[i][dir];
                    }
                }
                psbs_cache[dir] &= neighbour_tile.psbs;
                #[cfg(feature = "debug_verbose")]
                println!(
                    "{}'s psbs: {:?}, dir={})",
                    neighbours[dir], psbs_cache[dir], dir
                );
                if psbs_cache[dir].count_ones() == 0 {
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
        let hist = {
            if self.history.len() > self.retrace_strength as usize {
                for _ in 0..self.retrace_strength {
                    self.history.pop();
                }
                self.history.pop().unwrap()
            } else {
                let h = self.history[0].clone();
                self.history.clear();
                h
            }
        };
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
            tilemap.set(commands, index, &TileBuilder::new(texture_index));
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
        self.heap[heap_index].0 = tile.psbs.count_ones() as u8;
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
        // crate::debug::validate_heap(&self.heap, true);
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
    mut runner_query: Query<(Entity, &mut Tilemap, &mut WfcRunner), Without<AsyncWfcRunner>>,
) {
    runner_query
        .par_iter_mut()
        .for_each(|(entity, mut tilemap, mut runner)| {
            #[cfg(feature = "debug")]
            let start = std::time::SystemTime::now();
            let mut wfc_grid = WfcGrid::from_runner(&mut runner);
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
                    fallback(&mut c, entity, &tilemap, &runner);
                }
                c.entity(entity).remove::<WfcRunner>();
            })
        });
}

pub fn wave_function_collapse_async(
    commands: ParallelCommands,
    mut runner_query: Query<(
        Entity,
        &mut Tilemap,
        &mut WfcRunner,
        &AsyncWfcRunner,
        Option<&mut WfcGrid>,
    )>,
) {
    runner_query
        .par_iter_mut()
        .for_each(|(entity, mut tilemap, mut runner, _, wfc_grid)| {
            if let Some(mut grid) = wfc_grid {
                if grid.remaining > 0 && grid.retraced_time < grid.max_retrace_time {
                    let min_tile = grid.pop_min();
                    grid.collapse(min_tile.index);

                    if let Some(idx) = grid.get_tile(min_tile.index).unwrap().texture_index {
                        commands.command_scope(|mut c| {
                            tilemap.set(&mut c, min_tile.index, &TileBuilder::new(idx as u32));
                        })
                    }
                } else {
                    commands.command_scope(|mut c| {
                        if grid.retraced_time < grid.max_retrace_time {
                            // grid.apply_map(&mut c, &mut tilemap);
                        } else if let Some(fallback) = &grid.fallback {
                            fallback(&mut c, entity, &tilemap, &runner);
                        }
                        c.entity(entity).remove::<WfcRunner>();
                        c.entity(entity).remove::<AsyncWfcRunner>();
                        c.entity(entity).remove::<WfcGrid>();
                    });
                }
            } else {
                let mut grid = WfcGrid::from_runner(&mut runner);
                grid.collapse(grid.pick_random());
                commands.command_scope(|mut c| {
                    c.entity(entity).insert(grid);
                });
            }
        });
}
