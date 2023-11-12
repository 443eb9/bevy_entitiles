/// Direction order: up, right, left, down
use std::fs::File;

use bevy::prelude::{Commands, Component, Query, UVec2};
use rand::{distributions::Uniform, prelude::Distribution, rngs::StdRng, Rng, SeedableRng};
use ron::de::from_reader;
use serde::Serialize;

use crate::{math::FillArea, tilemap::Tilemap};

#[derive(Default, Clone, Copy)]
pub enum WfcMode {
    #[default]
    NonWeighted,
}

#[derive(Component)]
pub struct WaveFunctionCollapser {
    /// Defines what tiles can be placed next to each other using texture index.
    ///
    /// # Example
    ///
    /// `rule = [[...0100, ...0010, ...1001, ...1000], [...1000, ...0100, ...0101, ...0011]]`
    ///
    /// means tile with `texture_index = 0`'s left can be `2`, right can be `1`, up can be `0` or `3`, down can be `3`
    ///
    /// while tile with `texture_index = 1`'s left can be `3`, right can be `2`, up can be `2` or `0`, down can be `0` or `1`
    pub rule: Vec<[u128; 4]>,
    pub mode: WfcMode,
    pub seed: Option<u64>,
    pub area: FillArea,
    pub step_interval: Option<f32>,
}

impl WaveFunctionCollapser {
    /// The order of the directions are: up, right, left, down. If you don't know what format the config should be,
    /// plase check the comments for `rule` field in `WaveFunctionCollapser`.
    pub fn from_config(
        config_path: String,
        mode: WfcMode,
        area: FillArea,
        step_interval: Option<f32>,
        seed: Option<u64>,
    ) -> Self {
        let rdr = File::open(config_path.clone())
            .expect("Failed to load config file for wave function collapse");
        let rule: Vec<[u128; 4]> = match from_reader(rdr) {
            Ok(rule) => rule,
            Err(_) => panic!(
                "Failed to parse config file for wave function collapse: {}",
                config_path
            ),
        };

        if rule.len() > 128 {
            panic!("The length of the rule is too long. We currently only support rules with length <= 128");
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

#[derive(Clone)]
struct WfcTile {
    pub index: UVec2,
    pub collapsed: bool,
    pub texture_index: Option<u8>,
    pub heap_index: usize,
    pub psbs: u128,
    pub entropy: u8,
}

struct WfcHistory {
    index: UVec2,
    old_psbs: u128,
}

struct WfcGrid {
    mode: WfcMode,
    history: Vec<WfcHistory>,
    size: UVec2,
    grid: Vec<WfcTile>,
    max_psb: u8,
    rng: StdRng,
    rule: Vec<[u128; 4]>,
    heap: Vec<(u8, UVec2)>,
}

impl WfcGrid {
    pub fn from_collapser(collapser: &WaveFunctionCollapser) -> Self {
        let mut grid = Vec::with_capacity(collapser.area.size());
        let mut heap = Vec::with_capacity(collapser.area.size() + 1);
        // a placeholder
        heap.push((0, UVec2::new(0, 0)));
        let mut heap_index = 1;

        for y in 0..collapser.area.extent.y {
            for x in 0..collapser.area.extent.x {
                grid[(y * collapser.area.extent.x + x) as usize] = WfcTile {
                    heap_index,
                    index: UVec2 { x, y },
                    texture_index: None,
                    collapsed: false,
                    psbs: !0u128,
                    entropy: collapser.rule.len() as u8,
                };

                heap.push((collapser.rule.len() as u8, UVec2 { x, y }));
                heap_index += 1;
            }
        }

        WfcGrid {
            grid,
            size: collapser.area.extent,
            history: vec![],
            mode: collapser.mode,
            max_psb: collapser.rule.len() as u8,
            rule: collapser.rule.clone(),
            rng: match collapser.seed {
                Some(seed) => StdRng::seed_from_u64(seed),
                None => StdRng::from_entropy(),
            },
            heap,
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
        let min = self.heap.pop().unwrap();
        let min_tile = self.get_tile(min.1).unwrap().clone();

        self.heap[1] = min;
        self.get_tile_mut(self.heap[1].1).unwrap().heap_index = 1;
        self.shift_down(1);
        min_tile
    }

    pub fn collapse(&mut self, index: UVec2) {
        let Some(tile) = self.get_tile_mut(index) else {
            return;
        };
        if tile.collapsed {
            return;
        }
        tile.collapsed = true;

        // TODO collapse
        match self.mode {
            WfcMode::NonWeighted => {}
        }

        let old_psbs = tile.psbs;
        self.history.push(WfcHistory { index, old_psbs });

        self.spread(index);
    }

    pub fn spread(&mut self, center: UVec2) {
        let mut queue: Vec<UVec2> = vec![center];

        while !queue.is_empty() {
            let cur_ctr = queue.pop().unwrap();
            let cur_tile = self.get_tile(cur_ctr).unwrap();

            let Some(texture_index) = cur_tile.texture_index else {
                return;
            };

            let neighbours = self.neighbours(cur_ctr);
            let mut psbs_cache = [0u128; 4];

            for dir in 0..4 {
                let Some(neighbour_tile) = self.get_tile(neighbours[dir]) else {
                    return;
                };
                if neighbour_tile.collapsed {
                    return;
                }
                psbs_cache[dir] = neighbour_tile.psbs & self.rule[texture_index as usize][3 - dir];
            }

            for dir in 0..4 {
                self.get_tile_mut(neighbours[dir]).unwrap().psbs = psbs_cache[dir];
            }

            for neighbour in neighbours {
                self.update_entropy(neighbour);
            }
        }
    }

    pub fn retrace(&mut self) {
        // TODO retrace when collapse fails
    }

    pub fn random_texture(&mut self, psbs: u128) -> u8 {
        let range = Uniform::from(0..127);
        let mut rd_l = range.sample(&mut self.rng);
        let mut rd_r = rd_l;

        loop {
            if psbs & (1 << rd_l) != 0 {
                return rd_l;
            }
            if psbs & (1 << rd_r) != 0 {
                return rd_r;
            }
            rd_l -= 1;
            rd_r += 1;
        }
    }

    pub fn neighbours(&mut self, index: UVec2) -> [UVec2; 4] {
        [
            UVec2::new(index.x, index.y + 1),
            UVec2::new(index.x + 1, index.y),
            UVec2::new(index.x - 1, index.y),
            UVec2::new(index.x, index.y - 1),
        ]
    }

    fn update_entropy(&mut self, index: UVec2) {
        if self.is_out_of_grid(index) {
            return;
        }

        let tile = self.get_tile_mut(index).unwrap();
        tile.entropy = tile.psbs.count_ones() as u8;
        let heap_index = tile.heap_index;
        let entropy = tile.entropy;
        self.heap[heap_index].0 = entropy;
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
            let right = self.heap.get(index * 2 + 1).unwrap();
            if left.0 <= right.0 {
                left
            } else {
                right
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
}

pub fn wave_function_collapse(
    mut commands: Commands,
    mut collapser_query: Query<(&mut Tilemap, &WaveFunctionCollapser)>,
) {
    collapser_query
        .par_iter_mut()
        .for_each_mut(|(tilemap, collapser)| {
            let mut wfc_grid = WfcGrid::from_collapser(&collapser);
            wfc_grid.collapse(wfc_grid.pick_random());
        });
}
