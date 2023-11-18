/// Direction order: up, right, left, down
use std::fs::read_to_string;

use bevy::{
    ecs::entity::Entity,
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
    tilemap::{TileBuilder, Tilemap},
};

use super::{HeapElement, LookupHeap};

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
    step_interval: Option<f32>,
    max_retrace_factor: u32,
    max_retrace_count: u32,
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
    pub fn from_config(
        rule_path: String,
        area: FillArea,
        step_interval: Option<f32>,
        seed: Option<u64>,
    ) -> Self {
        let rule_vec: Vec<[Vec<u16>; 4]> =
            from_bytes(read_to_string(rule_path).unwrap().as_bytes()).unwrap();

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
            step_interval,
            seed,
            max_retrace_factor: size.ilog10().clamp(2, 16),
            max_retrace_count: size.ilog10().clamp(2, 16) * 100,
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

impl HeapElement for WfcTile {
    #[inline]
    fn set_index(&mut self, index: usize) {
        self.heap_index = index;
    }

    #[inline]
    fn get_index(&self) -> usize {
        self.heap_index
    }
}

#[derive(Clone)]
struct WfcHistory {
    lookup_heap: LookupHeap<usize, UVec2, WfcTile>,
    remaining: usize,
}

struct WfcGrid {
    mode: WfcMode,
    history: Vec<WfcHistory>,
    lookup_heap: LookupHeap<usize, UVec2, WfcTile>,
    size: UVec2,
    rng: StdRng,
    rule: Vec<[u128; 4]>,
    sampler: Option<Box<dyn Fn(&WfcTile, &mut StdRng) -> u8 + Send + Sync>>,
    remaining: usize,
    retrace_strength: u32,
    max_retrace_factor: u32,
    max_retrace_time: u32,
    retraced_time: u32,
    fallback: Option<Box<dyn Fn(&mut Commands, Entity, &Tilemap, &WfcRunner) + Send + Sync>>,
}

impl WfcGrid {
    pub fn from_runner(runner: &mut WfcRunner) -> Self {
        let mut lookup_heap = LookupHeap::new();
        let max_psbs = runner.rule.len();

        for y in 0..runner.area.extent.y {
            for x in 0..runner.area.extent.x {
                let index = UVec2 { x, y };
                lookup_heap.update_lookup(
                    index,
                    WfcTile {
                        heap_index: 0,
                        index: UVec2 { x, y },
                        texture_index: None,
                        collapsed: false,
                        psbs: (!0) >> (128 - max_psbs),
                    },
                );

                lookup_heap.insert_heap(max_psbs, index);
            }
        }

        WfcGrid {
            size: runner.area.extent,
            history: vec![],
            lookup_heap,
            mode: runner.mode.clone(),
            rule: runner.rule.clone(),
            rng: match runner.seed {
                Some(seed) => StdRng::seed_from_u64(seed),
                None => StdRng::from_entropy(),
            },
            remaining: runner.area.size(),
            retrace_strength: 1,
            max_retrace_factor: runner.max_retrace_factor,
            max_retrace_time: runner.max_retrace_count,
            retraced_time: 0,
            sampler: runner.sampler.take(),
            fallback: runner.fallback.take(),
        }
    }

    #[inline]
    pub fn get_tile(&self, index: UVec2) -> Option<&WfcTile> {
        self.lookup_heap.map_get(&index)
    }

    #[inline]
    pub fn get_tile_mut(&mut self, index: UVec2) -> Option<&mut WfcTile> {
        self.lookup_heap.map_get_mut(&index)
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
            lookup_heap: self.lookup_heap.clone(),
            remaining: self.remaining,
        };
        self.history.push(hist);

        let min_tile = self.lookup_heap.pop_min().unwrap();
        self.remaining -= 1;

        #[cfg(feature = "debug_verbose")]
        println!("popped: {}, remaining={}", min_tile.index, self.remaining);
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
        self.lookup_heap = hist.lookup_heap;
        self.remaining = hist.remaining;
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
        for tile in self.lookup_heap.lookup.values() {
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
        self.lookup_heap.heap[heap_index].unwrap().0 = tile.psbs.count_ones() as usize;
        self.lookup_heap.shift_up(heap_index);
    }

    #[cfg(feature = "debug_verbose")]
    fn print_grid(&self) {
        let mut result = "-------------------\n".to_string();
        let mut counter = 0;

        for t in self.lookup_heap.lookup.values() {
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
        crate::debug::validate_heap(&self.lookup_heap.heap, true);
        for i in 1..self.lookup_heap.heap.len() {
            assert_eq!(
                self.get_tile(self.lookup_heap.heap[i].unwrap().1)
                    .unwrap()
                    .heap_index,
                i,
                "heap index not match at: {}",
                self.lookup_heap.heap[i].unwrap().1
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
