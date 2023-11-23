use bevy::prelude::{UVec2, Vec2};

use crate::{
    render::extract::{ExtractedTilemap, ExtractedView}, tilemap::{tile::TileType, map::TilemapBuilder},
};

#[derive(Clone, Copy, Default, Debug)]
pub struct AabbBox2d {
    pub min: Vec2,
    pub max: Vec2,
}

impl AabbBox2d {
    pub fn new(min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> Self {
        AabbBox2d {
            min: Vec2::new(min_x, min_y),
            max: Vec2::new(max_x, max_y),
        }
    }

    pub fn from_chunk(chunk_index: UVec2, tilemap: &ExtractedTilemap) -> Self {
        match tilemap.tile_type {
            TileType::Square => {
                let chunk_render_size = tilemap.tile_render_size * tilemap.render_chunk_size as f32;
                AabbBox2d {
                    min: chunk_index.as_vec2() * chunk_render_size + tilemap.translation,
                    max: (chunk_index + 1).as_vec2() * chunk_render_size + tilemap.translation,
                }
            }
            TileType::IsometricDiamond => {
                let chunk_index = chunk_index.as_vec2();
                let chunk_render_size = tilemap.render_chunk_size as f32 * tilemap.tile_render_size;
                let center_x = (chunk_index.x - chunk_index.y) / 2. * chunk_render_size.x;
                let center_y = (chunk_index.x + chunk_index.y + 1.) / 2. * chunk_render_size.y;

                AabbBox2d {
                    min: Vec2::new(
                        center_x - chunk_render_size.x / 2. + tilemap.translation.x,
                        center_y - chunk_render_size.y / 2. + tilemap.translation.y,
                    ),
                    max: Vec2::new(
                        center_x + chunk_render_size.x / 2. + tilemap.translation.x,
                        center_y + chunk_render_size.y / 2. + tilemap.translation.y,
                    ),
                }
            }
        }
    }

    pub fn from_tilemap_builder(builder: &TilemapBuilder) -> AabbBox2d {
        match builder.tile_type {
            TileType::Square => {
                let tilemap_render_size = builder.size.as_vec2() * builder.tile_render_size;
                AabbBox2d {
                    min: (tilemap_render_size - tilemap_render_size) / 2. + builder.translation,
                    max: (tilemap_render_size + tilemap_render_size) / 2. + builder.translation,
                }
            }
            TileType::IsometricDiamond => {
                let size = builder.size.as_vec2();
                let tilemap_render_size = (size.x + size.y) / 2. * builder.tile_render_size;
                let center_x = (size.x - size.y) * builder.tile_render_size.x / 4.;
                let center_y = tilemap_render_size.y / 2.;
                AabbBox2d {
                    min: Vec2::new(
                        center_x - tilemap_render_size.x / 2. + builder.translation.x,
                        center_y - tilemap_render_size.y / 2. + builder.translation.y,
                    ),
                    max: Vec2::new(
                        center_x + tilemap_render_size.x / 2. + builder.translation.x,
                        center_y + tilemap_render_size.y / 2. + builder.translation.y,
                    ),
                }
            }
        }
    }

    pub fn from_camera(camera: &ExtractedView) -> Self {
        let half_width = camera.width * camera.scale;
        let half_height = camera.height * camera.scale;
        AabbBox2d {
            min: Vec2::new(
                camera.transform.x - half_width,
                camera.transform.y - half_height,
            ),
            max: Vec2::new(
                camera.transform.x + half_width,
                camera.transform.y + half_height,
            ),
        }
    }

    #[inline]
    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }

    #[inline]
    pub fn height(&self) -> f32 {
        self.max.y - self.min.y
    }

    #[inline]
    pub fn center(&self) -> Vec2 {
        (self.min + self.max) / 2.
    }

    #[inline]
    pub fn top_left(&self) -> Vec2 {
        Vec2::new(self.min.x, self.max.y)
    }

    #[inline]
    pub fn bottom_right(&self) -> Vec2 {
        Vec2::new(self.max.x, self.min.y)
    }

    #[inline]
    pub fn area(&self) -> f32 {
        self.width() * self.height()
    }

    #[inline]
    pub fn expand(&mut self, other: &AabbBox2d) {
        self.min = self.min.min(other.min);
        self.max = self.max.max(other.max);
    }

    #[inline]
    pub fn is_intersected(&self, other: &AabbBox2d) -> bool {
        self.min.x < other.max.x
            && self.max.x > other.min.x
            && self.min.y < other.max.y
            && self.max.y > other.min.y
    }

    #[inline]
    pub fn with_additional_translation(&self, translation: Vec2) -> Self {
        AabbBox2d {
            min: self.min + translation,
            max: self.max + translation,
        }
    }

    #[inline]
    pub fn with_uniform_scale(&self, scale: f32) -> Self {
        let width = self.width() * scale;
        let height = self.height() * scale;
        AabbBox2d {
            min: self.center() - Vec2::new(width, height) / 2.,
            max: self.center() + Vec2::new(width, height) / 2.,
        }
    }
}