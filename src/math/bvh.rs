use bevy::prelude::Res;

use crate::render::{chunk::RenderChunkStorage, extract::ExtractedTilemap};

use super::aabb::AabbBox2d;

pub struct TilemapBvhTree {
    root: BvhNode,
}

#[derive(Clone)]
pub struct BvhNode {
    pub aabb: AabbBox2d,
    pub left: Option<Box<BvhNode>>,
    pub right: Option<Box<BvhNode>>,
}

#[derive(Default, Debug, Clone)]
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
    pub fn new(tilemap_aabb: &AabbBox2d, aabbs: &Vec<AabbBox2d>, bucket_count: usize) -> Self {
        TilemapBvhTree {
            root: TilemapBvhTree::create_node_recursive(
                aabbs,
                &Bucket {
                    aabb_indices: (0..aabbs.len()).collect(),
                    aabb: *tilemap_aabb,
                },
                bucket_count,
            )
            .unwrap(),
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

        Some(Self::new(&tilemap.aabb, &processed_chunks, 10))
    }

    pub fn is_intersected_with(&self, other: &AabbBox2d) -> bool {
        true
    }

    #[allow(unconditional_recursion)]
    fn create_node_recursive(
        aabb_lookup: &Vec<AabbBox2d>,
        root_bucket: &Bucket,
        bucket_count: usize,
    ) -> Option<BvhNode> {
        if root_bucket.is_empty() {
            println!("Empty source");
            return None;
        }

        let (bucket_lhs, bucket_rhs) = TilemapBvhTree::divide(&root_bucket, aabb_lookup, bucket_count);

        println!("lhs: {:?},\nrhs: {:?} \n\n", bucket_lhs, bucket_rhs);

        let left_node =
            TilemapBvhTree::create_node_recursive(aabb_lookup, &bucket_lhs, bucket_count);
        let right_node =
            TilemapBvhTree::create_node_recursive(aabb_lookup, &bucket_rhs, bucket_count);

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

    // TODO: Optimization
    fn divide(
        bucket: &Bucket,
        aabb_lookup: &Vec<AabbBox2d>,
        bucket_count: usize,
    ) -> (Bucket, Bucket) {
        println!("dividing: {:?}", bucket);

        if bucket.aabb_indices.len() == 1 {
            return (Bucket::default(), Bucket::default());
        }

        if bucket.aabb_indices.len() == 2 {
            let bucket_a = Bucket {
                aabb_indices: vec![bucket.aabb_indices[0]],
                aabb: aabb_lookup[bucket.aabb_indices[0]],
            };
            let bucket_b = Bucket {
                aabb_indices: vec![bucket.aabb_indices[1]],
                aabb: aabb_lookup[bucket.aabb_indices[1]],
            };
            if bucket_a.aabb.center().x < bucket_b.aabb.center().x {
                return (bucket_a, bucket_b);
            } else {
                return (bucket_a, bucket_b);
            }
        }

        let parent_center = bucket.aabb.center();
        let half_parent_width = bucket.aabb.width() / 2.;
        let (bound_left, bound_right) = (
            parent_center.x - half_parent_width,
            parent_center.x + half_parent_width,
        );
        let bucket_width = (bound_right - bound_left) / bucket_count as f32;
        let (mut best_bucket_lhs, mut best_bucket_rhs) = (Bucket::default(), Bucket::default());
        let mut best_cost = f32::MAX;

        for strategy_index in 1..bucket_count {
            let bucket_split = bound_left + bucket_width * (strategy_index + 1) as f32;
            let (mut lhs, mut rhs) = (Bucket::default(), Bucket::default());

            for aabb_index in bucket.aabb_indices.iter() {
                if aabb_lookup[*aabb_index].center().x < bucket_split {
                    lhs.expand(&aabb_lookup[*aabb_index]);
                    lhs.insert(*aabb_index);
                } else {
                    rhs.expand(&aabb_lookup[*aabb_index]);
                    rhs.insert(*aabb_index);
                }
            }

            if lhs.is_empty() || rhs.is_empty() {
                println!("Invalid strategy: {:?}", strategy_index);
                continue;
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

mod test {
    use super::*;

    #[test]
    pub fn test_bvh() {
        let tree = serialize(&TilemapBvhTree::new(
            &AabbBox2d::new(-2., -2., 10., 6.),
            &vec![
                AabbBox2d::new(-1., 1., 1., 5.),
                AabbBox2d::new(1.5, -1., 3., 1.),
                AabbBox2d::new(2., 2., 4., 3.),
                AabbBox2d::new(5., 1., 8., 1.5),
                AabbBox2d::new(6., -1., 9., 1.25),
            ],
            10,
        ));
        println!("{}", tree);
    }

    fn serialize(tree: &TilemapBvhTree) -> String {
        serialize_recursive(Some(Box::new(tree.root.clone()))).unwrap()
    }

    fn serialize_recursive(root: Option<Box<BvhNode>>) -> Option<String> {
        let Some(root) = root else {
            return None;
        };

        let mut result = format!("{:?}\n", root.aabb);
        result += &format!(
            "left: {}\n",
            serialize_recursive(root.left).unwrap_or("Empty".to_string())
        )
        .to_string();
        result += &format!(
            "right: {}\n",
            serialize_recursive(root.right).unwrap_or("Empty".to_string())
        )
        .to_string();
        Some(result)
    }
}
