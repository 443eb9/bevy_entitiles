/// Direction order: up, right, left, down
use std::{collections::VecDeque, vec};

use bevy::{
    ecs::{entity::Entity, query::Without},
    math::Vec4,
    prelude::{Commands, Component, ParallelCommands, Query, UVec2},
    reflect::Reflect,
    utils::{HashMap, HashSet},
};
use rand::{distributions::WeightedIndex, rngs::StdRng, Rng, SeedableRng};

use crate::{
    math::{extension::TileIndex, TileArea},
    render::texture::TilemapTexture,
    serializing::{pattern::TilemapPattern, SerializedTile},
    tilemap::{
        layer::TileLayer,
        map::{Tilemap, TilemapBuilder},
        tile::{TileTexture, TileType},
    },
};

const DIR: [&'static str; 4] = ["up", "right", "left", "down"];
const HEX_DIR: [&'static str; 6] = [
    "up_right",
    "right",
    "down_right",
    "up_left",
    "left",
    "down_left",
];

#[derive(Reflect)]
pub struct WfcRules(pub Vec<Vec<u128>>);

impl WfcRules {
    pub fn from_file(rule_path: &str, tile_type: TileType) -> Self {
        let rule_vec: Vec<Vec<Vec<u8>>> =
            ron::from_str(std::fs::read_to_string(rule_path).unwrap().as_str()).unwrap();

        assert!(
            rule_vec.len() <= 128,
            "We only support 128 elements for now"
        );

        let mut rule_set = Vec::with_capacity(rule_vec.len());
        for tex_idx in 0..rule_vec.len() {
            let mut tex_rule: Vec<Vec<u8>> = {
                match tile_type {
                    TileType::Hexagonal(_) => vec![vec![]; 6],
                    _ => vec![vec![]; 4],
                }
            };
            for dir in 0..tex_rule.len() {
                for idx in rule_vec[tex_idx][dir].iter() {
                    tex_rule[dir].push(*idx);
                }
            }
            rule_set.push(tex_rule);
        }

        let mut rule = Vec::with_capacity(rule_set.len());
        for tex_idx in 0..rule_set.len() {
            let mut tex_rule = {
                match tile_type {
                    TileType::Hexagonal(_) => vec![0; 6],
                    _ => vec![0; 4],
                }
            };
            for dir in 0..tex_rule.len() {
                for idx in rule_set[tex_idx][dir].iter() {
                    tex_rule[dir] |= 1 << idx;
                }
            }
            rule.push(tex_rule);
        }

        let res = Self(rule);
        res.check_rules(tile_type);
        res
    }

    /// Check if there are conflicts in the rules.
    pub fn check_rules(&self, tile_type: TileType) {
        let (total_dirs, dir_names) = match tile_type {
            TileType::Hexagonal(_) => (6, HEX_DIR.to_vec()),
            _ => (4, DIR.to_vec()),
        };

        self.0.iter().enumerate().for_each(|(this_idx, elem)| {
            elem.iter().enumerate().for_each(|(dir, rule)| {
                (0..128).into_iter().for_each(|another_idx| {
                    if rule & (1 << another_idx) != 0 {
                        assert_ne!(
                            self.0[another_idx][total_dirs - dir - 1] & (1 << this_idx),
                            0,
                            "Conflict in rules! \
                            {}'s {} can be {}, but {}'s {} cannot be {}!",
                            this_idx,
                            dir_names[dir],
                            another_idx,
                            another_idx,
                            dir_names[total_dirs - dir - 1],
                            this_idx
                        );
                    }
                });
            });
        });
    }
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Reflect)]
pub enum WfcMode {
    #[default]
    /// Randomly pick one from the possibilities.
    NonWeighted,
    /// Pick one from the possibilities according to the weights.
    Weighted(Vec<u8>),
    /// You can use this to generate a map according to a noise function etc.
    CustomSampler,
}

#[derive(Component, Debug, Clone, Reflect)]
pub enum WfcSource {
    SingleTile(Vec<SerializedTile>),
    MapPattern(Vec<TilemapPattern>),
    MultiLayerMapPattern(UVec2, Vec<(Vec<TilemapPattern>, Option<TilemapTexture>)>),
}

impl WfcSource {
    /// Generate tiles with rules.
    ///
    /// The numbers you fill in the rules will be directly considered as the texture indices.
    pub fn from_texture_indices(conn_rules: &WfcRules) -> Self {
        let tiles = (0..conn_rules.0.len())
            .into_iter()
            .map(|r| SerializedTile {
                index: UVec2::ZERO,
                color: Vec4::ONE,
                texture: TileTexture::Static(vec![TileLayer::new().with_texture_index(r as u32)]),
            })
            .collect();
        Self::SingleTile(tiles)
    }

    /// Load tilemap patterns from the directory.
    ///
    /// The structure should looks like:
    /// ```
    /// C
    /// └── wfc_patterns
    ///     ├── wfc_pattern_0.ron
    ///     ├── wfc_pattern_1.ron
    ///     ..
    /// ```
    /// So the `directory`= `C:\\wfc_patterns`, `prefix` = `wfc_pattern_`.
    pub fn from_pattern_path(directory: String, prefix: String, conn_rules: &WfcRules) -> Self {
        let n = conn_rules.0.len();
        let mut patterns = Vec::with_capacity(n);

        for idx in 0..n {
            let serialized_pattern: TilemapPattern = ron::from_str(
                std::fs::read_to_string(format!("{}/{}{}.ron", directory, prefix, idx))
                    .unwrap()
                    .as_str(),
            )
            .unwrap();
            patterns.push(serialized_pattern);
        }

        Self::MapPattern(patterns)
    }
}

/// The order of the directions in config should be: up, right, left, down.
#[derive(Component, Reflect)]
pub struct WfcRunner {
    conn_rules: Vec<Vec<u128>>,
    mode: WfcMode,
    tile_type: TileType,
    sampler: Option<Box<dyn Fn(&WfcElement, &mut StdRng) -> u8 + Send + Sync>>,
    seed: Option<u64>,
    area: TileArea,
    max_retrace_factor: u32,
    max_retrace_time: u32,
    max_history: usize,
    fallback: Option<Box<dyn Fn(&mut Commands, Entity, &WfcRunner) + Send + Sync>>,
}

impl WfcRunner {
    pub fn new(tile_type: TileType, rules: WfcRules, area: TileArea, seed: Option<u64>) -> Self {
        let size = area.size();
        Self {
            conn_rules: rules.0,
            mode: WfcMode::NonWeighted,
            tile_type,
            sampler: None,
            area,
            seed,
            max_retrace_factor: size.ilog10().clamp(2, 16),
            max_retrace_time: size.ilog10().clamp(2, 16) * 100,
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
            ron::from_str(std::fs::read_to_string(weights_path).unwrap().as_str()).unwrap();
        assert_eq!(
            weights_vec.len(),
            self.conn_rules.len(),
            "weights length not match! weights: {}, rules: {}",
            weights_vec.len(),
            self.conn_rules.len()
        );
        self.mode = WfcMode::Weighted(weights_vec);
        self
    }

    /// Set the custom sampler function.
    /// The function should accept `WfcTile`,`StdRng` and return a `u8` as the texture index.
    pub fn with_custom_sampler(
        mut self,
        custom_sampler: Box<dyn Fn(&WfcElement, &mut StdRng) -> u8 + Send + Sync>,
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
        max_retrace_time: Option<u32>,
    ) -> Self {
        if let Some(factor) = max_retrace_factor {
            assert!(factor <= 16, "max_retrace_factor should be <= 16");
            self.max_retrace_factor = factor;
        }
        if let Some(time) = max_retrace_time {
            self.max_retrace_time = time;
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
        fallback: Box<dyn Fn(&mut Commands, Entity, &WfcRunner) + Send + Sync>,
    ) -> Self {
        self.fallback = Some(fallback);
        self
    }

    /// Get the rule for wfc.
    pub fn get_rule(&self) -> &Vec<Vec<u128>> {
        &self.conn_rules
    }
}

#[derive(Component, Reflect)]
pub struct WfcData {
    pub(crate) data: Vec<u8>,
    pub(crate) size: UVec2,
}

impl WfcData {
    pub(crate) fn new(size: UVec2) -> Self {
        Self {
            data: vec![0; (size.x * size.y) as usize],
            size,
        }
    }

    pub fn get(&self, index: UVec2) -> Option<u8> {
        self.data
            .get((index.y * self.size.x + index.x) as usize)
            .cloned()
    }

    pub(crate) fn set(&mut self, index: UVec2, value: u8) {
        self.data[(index.y * self.size.x + index.x) as usize] = value;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub struct WfcElement {
    pub index: UVec2,
    pub collapsed: bool,
    pub element_index: Option<u8>,
    pub psbs: u128,
}

impl WfcElement {
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

#[derive(Clone, Reflect)]
pub struct WfcHistory {
    uncollapsed: HashSet<(u8, UVec2)>,
    elements: HashMap<UVec2, WfcElement>,
    remaining: usize,
}

#[derive(Component)]
pub struct WfcGrid {
    mode: WfcMode,
    tile_type: TileType,
    area: TileArea,
    rng: StdRng,
    conn_rules: Vec<Vec<u128>>,
    uncollapsed: HashSet<(u8, UVec2)>,
    elements: HashMap<UVec2, WfcElement>,
    remaining: usize,
    history: Vec<Option<WfcHistory>>,
    cur_hist: usize,
    retrace_strength: u32,
    max_retrace_factor: u32,
    max_retrace_time: u32,
    retraced_time: u32,
    sampler: Option<Box<dyn Fn(&WfcElement, &mut StdRng) -> u8 + Send + Sync>>,
    fallback: Option<Box<dyn Fn(&mut Commands, Entity, &WfcRunner) + Send + Sync>>,
}

impl WfcGrid {
    pub fn from_runner(runner: &mut WfcRunner) -> Self {
        let mut uncollapsed = HashSet::new();
        let mut elements = HashMap::new();
        let max_psbs = runner.conn_rules.len() as u8;

        for y in 0..runner.area.extent.y {
            for x in 0..runner.area.extent.x {
                elements.insert(
                    UVec2 { x, y },
                    WfcElement {
                        index: UVec2 { x, y },
                        element_index: None,
                        collapsed: false,
                        psbs: (!0) >> (128 - max_psbs),
                    },
                );

                uncollapsed.insert((max_psbs, UVec2 { x, y }));
            }
        }

        WfcGrid {
            mode: runner.mode.clone(),
            area: runner.area,
            conn_rules: runner.conn_rules.clone(),
            uncollapsed,
            elements,
            history: vec![None; runner.max_history],
            cur_hist: 0,
            tile_type: runner.tile_type,
            rng: match runner.seed {
                Some(seed) => StdRng::seed_from_u64(seed),
                None => StdRng::from_entropy(),
            },
            remaining: runner.area.size(),
            retrace_strength: 1,
            max_retrace_factor: runner.max_retrace_factor,
            max_retrace_time: runner.max_retrace_time,
            retraced_time: 0,
            sampler: runner.sampler.take(),
            fallback: runner.fallback.take(),
        }
    }

    pub fn collapse(&mut self) {
        self.history[self.cur_hist] = Some(WfcHistory {
            uncollapsed: self.uncollapsed.clone(),
            elements: self.elements.clone(),
            remaining: self.remaining,
        });
        self.cur_hist = (self.cur_hist + 1) % self.history.len();

        let min = self.get_min();
        let elem = self.elements.get_mut(&min).unwrap();
        self.uncollapsed
            .remove(&(elem.psbs.count_ones() as u8, elem.index));

        let psb = match &self.mode {
            WfcMode::NonWeighted => {
                let psb_vec = elem.get_psbs_vec();
                psb_vec[self.rng.gen_range(0..psb_vec.len())]
            }
            WfcMode::Weighted(w) => {
                let psb_vec = elem.get_psbs_vec();
                let weights = psb_vec.iter().map(|p| w[*p as usize]).collect::<Vec<_>>();
                psb_vec[self.rng.sample(WeightedIndex::new(weights).unwrap())]
            }
            WfcMode::CustomSampler => {
                let mut rng = self.rng.clone();
                let res = self.sampler.as_ref().unwrap()(&elem, &mut rng) as u8;
                self.rng = rng;
                res
            }
        };

        elem.element_index = Some(psb);
        elem.psbs = 1 << psb;
        elem.collapsed = true;
        self.remaining -= 1;

        self.retrace_strength *= self.rng.gen_range(1..=self.max_retrace_factor);

        let index = elem.index;
        self.constrain(index);
    }

    pub fn constrain(&mut self, center: UVec2) {
        let mut queue = VecDeque::from([center]);
        let mut spreaded = HashSet::from([center]);

        while !queue.is_empty() {
            let cur_center = queue.pop_front().unwrap();
            spreaded.insert(cur_center);

            let cur_elem = self.elements.get(&cur_center).cloned().unwrap();
            let neis = cur_center.neighbours(self.tile_type, false);
            let nei_count = neis.len();

            for dir in 0..nei_count {
                let Some(nei_index) = neis[dir] else {
                    continue;
                };
                let Some(nei_elem) = self.elements.get_mut(&nei_index) else {
                    continue;
                };
                if nei_elem.collapsed || spreaded.contains(&nei_index) {
                    continue;
                }

                let mut psb = 0;
                let psb_rec = nei_elem.psbs;
                cur_elem.get_psbs_vec().into_iter().for_each(|p| {
                    psb |= self.conn_rules[p as usize][dir];
                });
                nei_elem.psbs &= psb;

                if nei_elem.psbs.count_ones() == 0 {
                    self.retrace();
                    return;
                }

                if nei_elem.psbs != psb_rec {
                    queue.push_back(nei_index);
                    let new_psbs = nei_elem.psbs.count_ones() as u8;
                    self.update_entropy(psb_rec.count_ones() as u8, new_psbs as u8, nei_index);
                }
            }
        }

        self.retrace_strength = 1;
    }

    pub fn update_entropy(&mut self, old: u8, new: u8, target: UVec2) {
        self.uncollapsed.remove(&(old, target));
        self.uncollapsed.insert((new, target));
    }

    pub fn retrace(&mut self) {
        let hist = {
            let hist_len = self.history.len();
            let strength = self.retrace_strength as usize;

            if hist_len <= strength {
                // max retrace time exceeded
                self.retraced_time = self.max_retrace_time;
            } else {
                if self.cur_hist >= strength {
                    self.cur_hist -= strength;
                } else {
                    // need to wrap around
                    let hist_to_be = hist_len - (strength - self.cur_hist);
                    if self.history[hist_to_be].is_none() {
                        // retrace failed
                        self.retraced_time = self.max_retrace_time;
                    } else {
                        self.cur_hist = hist_to_be;
                    }
                }
            }

            // in case the cur_hist is 0
            self.history[(self.cur_hist + hist_len - 1) % hist_len]
                .clone()
                .unwrap()
        };

        self.remaining = hist.remaining;
        self.uncollapsed = hist.uncollapsed;
        self.elements = hist.elements;
        self.retraced_time += self.retrace_strength;
    }

    pub fn get_min(&mut self) -> UVec2 {
        let mut min_entropy = u8::MAX;
        let mut candidates = Vec::with_capacity(self.remaining);
        self.uncollapsed.iter().for_each(|(entropy, index)| {
            if entropy < &min_entropy {
                min_entropy = *entropy;
                candidates.clear();
                candidates.push(*index);
            } else if entropy == &min_entropy {
                candidates.push(*index);
            }
        });
        candidates[self.rng.gen_range(0..candidates.len())]
    }

    pub fn apply_data(&mut self, commands: &mut Commands, entity: Entity) {
        let mut data = WfcData::new(self.area.extent);
        self.elements.drain().for_each(|(i, e)| {
            data.set(i, e.element_index.unwrap());
        });
        commands.entity(entity).insert(data);
    }
}

pub fn wave_function_collapse(
    commands: ParallelCommands,
    mut runner_query: Query<(Entity, &mut WfcRunner), Without<WfcData>>,
) {
    runner_query
        .par_iter_mut()
        .for_each(|(entity, mut runner)| {
            let mut wfc_grid = WfcGrid::from_runner(&mut runner);

            while wfc_grid.remaining > 0 && wfc_grid.retraced_time < wfc_grid.max_retrace_time {
                wfc_grid.collapse();
            }

            commands.command_scope(|mut c| {
                if wfc_grid.retraced_time < wfc_grid.max_retrace_time {
                    wfc_grid.apply_data(&mut c, entity);
                } else if let Some(fallback) = wfc_grid.fallback {
                    fallback(&mut c, entity, &runner);
                    c.entity(entity).remove::<WfcRunner>();
                }
            });
        });
}

pub fn wfc_applier(
    commands: ParallelCommands,
    mut tilemaps_query: Query<(
        Entity,
        Option<&mut Tilemap>,
        &WfcData,
        &WfcRunner,
        &WfcSource,
    )>,
) {
    tilemaps_query
        .par_iter_mut()
        .for_each(|(entity, mut tilemap, wfc_data, runner, source)| {
            commands.command_scope(|mut c| {
                match source {
                    WfcSource::SingleTile(tiles) => {
                        for (index, tex) in wfc_data.data.iter().enumerate() {
                            let ser_tile = tiles.get(*tex as usize).unwrap();
                            tilemap.as_mut().unwrap().set(
                                &mut c,
                                UVec2 {
                                    x: index as u32 % wfc_data.size.x,
                                    y: index as u32 / wfc_data.size.x,
                                },
                                ser_tile.clone().into(),
                            );
                        }
                    }
                    WfcSource::MapPattern(patterns) => {
                        wfc_data.data.iter().enumerate().for_each(|(i, e)| {
                            let p = &patterns[*e as usize];
                            p.apply(
                                &mut c,
                                (UVec2 {
                                    x: i as u32 % runner.area.extent.x,
                                    y: i as u32 / runner.area.extent.x,
                                } + runner.area.origin)
                                    * p.size,
                                &mut tilemap.as_mut().unwrap(),
                            );
                        });
                    }
                    WfcSource::MultiLayerMapPattern(size, patterns) => {
                        wfc_data.data.iter().enumerate().for_each(|(i, e)| {
                            let (p, tex) = &patterns[*e as usize];
                            p.iter().for_each(|layer| {
                                let mut new_layer = TilemapBuilder::new(
                                    TileType::Square,
                                    *size,
                                    size.as_vec2(),
                                    layer.label.clone().unwrap(),
                                );
                                new_layer.with_translation(
                                    (UVec2 {
                                        x: i as u32 % runner.area.extent.x,
                                        y: i as u32 / runner.area.extent.x,
                                    } * *size
                                        * *size)
                                        .as_vec2(),
                                );
                                if let Some(texture) = tex {
                                    new_layer.with_texture(texture.clone());
                                }
                                let mut tilemap = new_layer.build(&mut c);
                                layer.apply(&mut c, UVec2::ZERO, &mut tilemap);
                                c.entity(tilemap.id).insert(tilemap);
                                // c.entity(entity).despawn_recursive();
                                return;
                            });
                        });
                    }
                }

                let mut commands = c.entity(entity);
                commands.remove::<WfcData>();
                commands.remove::<WfcSource>();
                commands.remove::<WfcRunner>();
            });
        });
}
