use bevy::{
    asset::{Assets, Handle},
    ecs::{
        query::With,
        system::{Commands, Query, ResMut, Resource},
    },
    math::Vec2,
    prelude::{Image, UVec2},
    render::{
        render_resource::{FilterMode, Sampler},
        texture::GpuImage,
    },
    utils::HashMap,
};

use crate::tilemap::map::{Tilemap, WaitForTextureUsageChange};

/// Notice that the UVs are not the one you might think.
/// They are pixel coordinates instead of normalized coordinates.
#[derive(Clone, Debug, PartialEq)]
pub struct TileUV {
    pub(crate) min: Vec2,
    pub(crate) max: Vec2,
}

impl TileUV {
    #[inline]
    pub fn btm_left(&self) -> Vec2 {
        self.min
    }

    #[inline]
    pub fn btm_right(&self) -> Vec2 {
        Vec2 {
            x: self.max.x as f32,
            y: self.min.y as f32,
        }
    }

    #[inline]
    pub fn top_left(&self) -> Vec2 {
        Vec2 {
            x: self.min.x as f32,
            y: self.max.y as f32,
        }
    }

    #[inline]
    pub fn top_right(&self) -> Vec2 {
        self.max
    }

    #[inline]
    pub fn render_size(&self) -> Vec2 {
        self.max - self.min
    }
}

impl From<(UVec2, UVec2)> for TileUV {
    #[inline]
    fn from(value: (UVec2, UVec2)) -> Self {
        Self {
            min: value.0.as_vec2(),
            max: value.1.as_vec2(),
        }
    }
}

#[derive(Resource, Default)]
pub struct TilemapTexturesStorage {
    textures: HashMap<Handle<Image>, GpuImage>,
}

impl TilemapTexturesStorage {
    pub fn insert(&mut self, handle: Handle<Image>, mut gpu_image: GpuImage, sampler: Sampler) {
        gpu_image.sampler = sampler;
        self.textures.insert(handle, gpu_image);
    }

    pub fn get(&self, handle: &Handle<Image>) -> Option<&GpuImage> {
        self.textures.get(handle)
    }

    pub fn contains(&self, handle: &Handle<Image>) -> bool {
        self.textures.contains_key(handle)
    }
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct TilemapTextureDescriptor {
    pub size: UVec2,
    pub tiles_uv: Vec<TileUV>,
    pub filter_mode: FilterMode,
    /// Be honest please :)
    pub is_uniform: bool,
}

impl TilemapTextureDescriptor {
    /// Creates a new uniform descriptor from a full grid of tiles. The texture should be filled with tiles.
    /// Just like the one in the example.
    ///
    /// Use `TileUVBuilder` to create a non-uniform descriptor.
    pub fn from_full_grid(size: UVec2, tile_count: UVec2, filter_mode: FilterMode) -> Self {
        assert_eq!(
            (size % tile_count),
            UVec2::ZERO,
            "The texture size must be a multiple of the tile count."
        );

        let mut tiles_uv = Vec::with_capacity((tile_count.x * tile_count.y) as usize);
        let unit_uv = (size / tile_count).as_vec2();

        for y in 0..tile_count.y {
            for x in 0..tile_count.x {
                tiles_uv.push(TileUV {
                    min: Vec2 {
                        x: unit_uv.x * x as f32,
                        y: unit_uv.y * y as f32,
                    },
                    max: Vec2 {
                        x: unit_uv.x * (x + 1) as f32,
                        y: unit_uv.y * (y + 1) as f32,
                    },
                });
            }
        }

        Self {
            size,
            tiles_uv,
            filter_mode,
            is_uniform: true,
        }
    }
}

pub fn set_texture_usage(
    mut commands: Commands,
    mut tilemaps_query: Query<&mut Tilemap, With<WaitForTextureUsageChange>>,
    mut image_assets: ResMut<Assets<Image>>,
) {
    for mut tilemap in tilemaps_query.iter_mut() {
        tilemap.set_usage(&mut commands, &mut image_assets);
    }
}
