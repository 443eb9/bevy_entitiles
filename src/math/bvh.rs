use bevy::prelude::Res;

use crate::render::{chunk::RenderChunkStorage, extract::ExtractedTilemap};

use super::geometry::AabbBox2d;

pub struct TilemapBvhTree {
    root: BvhNode,
}

pub struct BvhNode {
    pub aabb: AabbBox2d,
    pub left: Option<Box<BvhNode>>,
    pub right: Option<Box<BvhNode>>,
}

#[derive(Default)]
struct Bucket {
    aabb_indices: Vec<usize>,
    aabb: AabbBox2d,
}

impl Bucket {
    pub fn insert(&mut self, aabb_index: usize) {
        self.aabb_indices.push(aabb_index);
    }

    pub fn expand(&mut self, aabb: &AabbBox2d) {
        if self.aabb_indices.is_empty() {
            self.aabb = aabb.clone();
        } else {
            self.aabb.expand(aabb);
        }
    }

    pub fn cost(&self) -> f32 {
        self.aabb.area()
    }

    pub fn is_empty(&self) -> bool {
        self.aabb_indices.is_empty()
    }
}

impl TilemapBvhTree {
    pub fn new(
        tilemap_aabb: &AabbBox2d,
        root: &AabbBox2d,
        aabbs: &Vec<AabbBox2d>,
        bucket_size: f32,
    ) -> Self {
        let root_node = BvhNode {
            aabb: root.clone(),
            left: None,
            right: None,
        };

        TilemapBvhTree::create_node_recursive(
            aabbs,
            &Bucket {
                aabb_indices: (0..aabbs.len()).collect(),
                aabb: *tilemap_aabb,
            },
            bucket_size,
        );

        TilemapBvhTree {
            root: root_node,
        }
    }

    pub fn from_tilemap(
        tilemap: &ExtractedTilemap,
        render_chunk_storage: &Res<RenderChunkStorage>,
    ) -> Option<Self> {
        let chunks = render_chunk_storage.get(tilemap.id)?;
        let mut processed_chunks = Vec::with_capacity(chunks.len());
        for chunk in chunks.iter() {
            if let Some(c) = chunk {
                processed_chunks.push(c.aabb);
            }
        }

        Some(Self::new(
            &tilemap.aabb,
            &tilemap.aabb,
            &processed_chunks,
            100.,
        ))
    }

    pub fn is_intersected_with(&self, other: &AabbBox2d) -> bool {
        true
    }

    #[allow(unconditional_recursion)]
    fn create_node_recursive(
        aabb_lookup: &Vec<AabbBox2d>,
        root_bucket: &Bucket,
        bucket_size: f32,
    ) -> Option<BvhNode> {
        let (bucket_lhs, bucket_rhs) = TilemapBvhTree::divide(
            &root_bucket.aabb,
            &root_bucket.aabb_indices,
            aabb_lookup,
            (root_bucket.aabb.width() / bucket_size).ceil() as usize,
        );

        let left_node =
            TilemapBvhTree::create_node_recursive(aabb_lookup, &bucket_lhs, bucket_size);
        let right_node =
            TilemapBvhTree::create_node_recursive(aabb_lookup, &bucket_rhs, bucket_size);

        if left_node.is_none() && right_node.is_none() {
            return None;
        }

        Some(BvhNode {
            aabb: root_bucket.aabb.clone(),
            left: {
                if let Some(node) = left_node {
                    Some(Box::new(node))
                } else {
                    None
                }
            },
            right: {
                if let Some(node) = right_node {
                    Some(Box::new(node))
                } else {
                    None
                }
            },
        })
    }

    fn divide(
        parent_aabb: &AabbBox2d,
        child_aabbs: &Vec<usize>,
        aabb_lookup: &Vec<AabbBox2d>,
        bucket_count: usize,
    ) -> (Bucket, Bucket) {
        let (bound_left, bound_right) = (
            parent_aabb.center.x - parent_aabb.width / 2.,
            parent_aabb.center.x + parent_aabb.width / 2.,
        );
        let bucket_width = (bound_right - bound_left) / bucket_count as f32;
        let (mut best_bucket_lhs, mut best_bucket_rhs) = (Bucket::default(), Bucket::default());
        let mut best_cost = f32::MAX;

        for strategy_index in 1..bucket_count {
            let bucket_split = bound_left + bucket_width * (strategy_index + 1) as f32;
            let (mut lhs, mut rhs) = (Bucket::default(), Bucket::default());

            for aabb_index in child_aabbs.iter() {
                if aabb_lookup[*aabb_index].center.x < bucket_split {
                    lhs.insert(*aabb_index);
                    lhs.expand(&aabb_lookup[*aabb_index]);
                } else {
                    rhs.insert(*aabb_index);
                    rhs.expand(&aabb_lookup[*aabb_index]);
                }
            }

            if lhs.cost() + rhs.cost() < best_cost {
                best_cost = lhs.cost() + rhs.cost();
                best_bucket_lhs = lhs;
                best_bucket_rhs = rhs;
            }
        }

        (best_bucket_lhs, best_bucket_rhs)
    }
}
