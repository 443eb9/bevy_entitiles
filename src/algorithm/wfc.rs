/// Direction order: up, right, left, down
use std::fs::read_to_string;

use bevy::{
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

        println!("rule: {:?}", rule);

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
    pub entropy: u16,
}

struct WfcHistory {
    index: UVec2,
    old_psbs: IndexSet<u16>,
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
                    entropy: collapser.rule.len() as u16,
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
        let Some(tile) = self.get_tile(index) else {
            return;
        };
        if tile.collapsed {
            return;
        }
        if tile.psbs.len() == 0 {
            println!("start retrace because of: {}", index);
            self.retrace();
            return;
        }

        println!("collasping: {:?}", index);
        self.print_grid();

        let rd = match self.mode {
            WfcMode::NonWeighted => self.rng.sample(Uniform::new(0, tile.entropy as usize)),
        };

        let tile = self.get_tile_mut(index).unwrap();
        tile.texture_index = Some(tile.psbs[rd]);
        tile.collapsed = true;

        let old_psbs = tile.psbs.clone();
        tile.psbs = IndexSet::from([tile.psbs[rd]]);
        tile.entropy = 1;
        let index = tile.index;
        self.history.push(WfcHistory { index, old_psbs });
        self.remaining -= 1;

        self.spread_constraint(index);
    }

    pub fn spread_constraint(&mut self, center: UVec2) {
        let mut queue: Vec<UVec2> = vec![center];
        let mut spreaded = HashSet::default();

        while !queue.is_empty() {
            let cur_ctr = queue.pop().unwrap();
            spreaded.insert(cur_ctr);
            let cur_tile = self.get_tile(cur_ctr).unwrap().clone();

            if cur_tile.entropy == self.max_psbs || cur_tile.collapsed {
                continue;
            }

            let neighbours = self.neighbours(cur_ctr);
            let mut psbs_cache = vec![IndexSet::default(); 4];

            // constrain
            for dir in 0..4 {
                let Some(neighbour_tile) = self.get_tile(neighbours[dir]) else {
                    return;
                };
                if neighbour_tile.collapsed || spreaded.contains(&neighbours[dir]) {
                    return;
                }
                psbs_cache[dir] = neighbour_tile.psbs.clone();

                for psb in cur_tile.psbs.iter() {
                    psbs_cache[dir] = psbs_cache[dir]
                        .intersection(&self.rule[*psb as usize][3 - dir])
                        .map(|e| *e)
                        .collect::<IndexSet<u16>>();
                }
            }

            for dir in 0..4 {
                self.get_tile_mut(neighbours[dir]).unwrap().psbs = psbs_cache[dir].clone();
            }

            for neighbour in neighbours {
                self.update_entropy(neighbour);
            }
        }
    }

    pub fn retrace(&mut self) {
        for _ in 0..2 {
            let hist = self.history.pop().unwrap();
            self.get_tile_mut(hist.index).unwrap().psbs = hist.old_psbs;
            self.update_entropy(hist.index);
            self.remaining += 1;
        }
    }

    pub fn apply_map(&self, commands: &mut Commands, tilemap: &mut Tilemap) {
        println!("map collapsed: {:?}", self.grid);
        for tile in self.grid.iter() {
            let index = tile.index;
            let texture_index = tile.texture_index.unwrap() as u32;
            tilemap.set(commands, TileBuilder::new(index, texture_index));
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
        tile.entropy = tile.psbs.len() as u16;
        let heap_index = tile.heap_index;
        let entropy = tile.entropy;
        self.heap[heap_index].0 = entropy;
        self.shift_up(heap_index);
        self.shift_down(heap_index);
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
        let mut result = "================\n".to_string();
        let mut counter = 0;

        for i in 0..self.grid.len() {
            result.push_str(&format!(
                "{:?}({})",
                self.grid[i].texture_index, self.grid[i].entropy
            ));
            counter += 1;
            if counter == self.size.x {
                result.push_str("\n");
                counter = 0;
            }
        }

        result.push_str("================");
        println!("{}", result);
    }
}

pub fn wave_function_collapse(
    commands: ParallelCommands,
    mut collapser_query: Query<(&mut Tilemap, &WaveFunctionCollapser)>,
) {
    collapser_query
        .par_iter_mut()
        .for_each_mut(|(mut tilemap, collapser)| {
            let mut wfc_grid = WfcGrid::from_collapser(&collapser);
            wfc_grid.print_grid();
            wfc_grid.collapse(wfc_grid.pick_random());

            while wfc_grid.remaining > 0 {
                let min_tile = wfc_grid.pop_min();
                wfc_grid.collapse(min_tile.index);
            }

            commands.command_scope(|mut c| {
                wfc_grid.apply_map(&mut c, &mut tilemap);
            })
        });
}
