// use rbtree::RBTree;

pub mod pathfinding;
pub mod wfc;

// pub(crate) struct MultiRbTree<K, V>
// where
//     K: Ord + Copy,
// {
//     pub tree: RBTree<K, Vec<V>>,
// }

// impl<K, V> MultiRbTree<K, V>
// where
//     K: Ord + Copy,
// {
//     pub fn new() -> Self {
//         Self {
//             tree: RBTree::new(),
//         }
//     }

//     pub fn insert(&mut self, key: K, value: V) {
//         if let Some(values) = self.tree.get_mut(&key) {
//             values.push(value);
//         } else {
//             self.tree.insert(key, vec![value]);
//         }
//     }

//     pub fn pop_first(&mut self) -> Option<(K, V)> {
//         let (key, mut values) = self.tree.pop_first()?;
//         let value = values.pop()?;
//         if !values.is_empty() {
//             self.tree.insert(key, values);
//         }
//         Some((key, value))
//     }

//     pub fn contains_key(&self, key: K) -> bool {
//         self.tree.contains_key(&key)
//     }
// }
