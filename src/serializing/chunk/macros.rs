#[macro_export]
macro_rules! impl_chunk_saver {
    ($ty:ty) => {
        impl $ty {
            /// For example if path = C:\\maps, then the crate will create:
            /// ```
            /// C
            /// └── maps
            ///     └── (your tilemap's name)
            ///         ├── tile_chunks
            ///         │   ├── 0_0.ron
            ///         │   ├── 0_1.ron
            ///         │   ...
            ///         ├── path_tile_chunks
            ///         ...
            /// ```
            pub fn new(path: String) -> Self {
                Self {
                    path,
                    chunks: vec![],
                    remove_after_save: false,
                    progress: 0,
                    cpf: 1,
                }
            }
        
            pub fn with_single(mut self, chunk_index: IVec2) -> Self {
                self.chunks.push(chunk_index);
                self
            }
        
            pub fn with_range(mut self, start_index: IVec2, end_index: IVec2) -> Self {
                assert!(
                    start_index.x <= end_index.x && start_index.y <= end_index.y,
                    "start_index({}) must be less than (or equal to) end_index({})!",
                    start_index,
                    end_index
                );
        
                self.chunks
                    .extend((start_index.y..=end_index.y).into_iter().flat_map(|y| {
                        (start_index.x..=end_index.x)
                            .into_iter()
                            .map(move |x| IVec2 { x, y })
                    }));
                self
            }
        
            pub fn with_multiple_ranges(mut self, ranges: Vec<IAabb2d>) -> Self {
                self.chunks
                    .extend(ranges.iter().flat_map(|aabb| (*aabb).into_iter()));
                self
            }
        
            pub fn remove_after_save(mut self) -> Self {
                self.remove_after_save = true;
                self
            }

            pub fn with_chunks_per_frame(mut self, chunks_per_frame: usize) -> Self {
                self.cpf = chunks_per_frame;
                self
            }
        }
    };
}
