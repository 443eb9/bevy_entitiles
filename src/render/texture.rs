use bevy::{
    asset::{Assets, Handle},
    ecs::{
        query::With,
        system::{Commands, Query, ResMut, Resource},
    },
    math::Vec2,
    prelude::{Image, UVec2},
    render::{render_resource::{FilterMode, Sampler}, texture::GpuImage},
    utils::HashMap,
};

use crate::tilemap::map::{Tilemap, WaitForTextureUsageChange};

#[derive(Clone, Debug, PartialEq)]
pub struct TileUV {
    pub min: Vec2,
    pub max: Vec2,
}

impl TileUV {
    #[inline]
    pub fn btm_left(&self) -> Vec2 {
        self.min
    }

    #[inline]
    pub fn btm_right(&self) -> Vec2 {
        Vec2 {
            x: self.max.x,
            y: self.min.y,
        }
    }

    #[inline]
    pub fn top_left(&self) -> Vec2 {
        Vec2 {
            x: self.min.x,
            y: self.max.y,
        }
    }

    #[inline]
    pub fn top_right(&self) -> Vec2 {
        self.max
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
    pub tiles_uv: Vec<TileUV>,
    pub filter_mode: FilterMode,
}

impl TilemapTextureDescriptor {
    /// Creates a new descriptor from a full grid of tiles. The texture should be filled with tiles.
    /// Just like the one in the example.
    pub fn from_full_grid(tile_count: UVec2, filter_mode: FilterMode) -> Self {
        let mut tiles_uv = Vec::with_capacity((tile_count.x * tile_count.y) as usize);
        let unit_uv = 1. / tile_count.as_vec2();

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
            tiles_uv,
            filter_mode,
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
