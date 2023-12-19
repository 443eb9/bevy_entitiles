use bevy::{prelude::{UVec2, Vec2}, reflect::Reflect};

use crate::{
    render::extract::{ExtractedTilemap, ExtractedView},
    tilemap::{map::TilemapBuilder, tile::TileType},
};

#[derive(Clone, Copy, Default, Debug, Reflect)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
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
        let pivot_offset = tilemap.pivot * tilemap.tile_slot_size;

        match tilemap.tile_type {
            TileType::Square => {
                let chunk_render_size = tilemap.tile_slot_size * tilemap.render_chunk_size as f32;
                tilemap.transform.transform_aabb(AabbBox2d {
                    min: chunk_index.as_vec2() * chunk_render_size - pivot_offset,
                    max: (chunk_index + 1).as_vec2() * chunk_render_size - pivot_offset,
                })
            }
            TileType::Isometric => {
                let chunk_index = chunk_index.as_vec2();
                let half_chunk_render_size =
                    tilemap.render_chunk_size as f32 * tilemap.tile_slot_size / 2.;
                let center_x = (chunk_index.x - chunk_index.y) * half_chunk_render_size.x;
                let center_y = (chunk_index.x + chunk_index.y + 1.) * half_chunk_render_size.y;
                let center = Vec2 {
                    x: center_x,
                    y: center_y,
                } - pivot_offset;

                tilemap.transform.transform_aabb(AabbBox2d {
                    min: center - half_chunk_render_size,
                    max: center + half_chunk_render_size,
                })
            }
            TileType::Hexagonal(c) => {
                /*
                 * MATHEMATICAL MAGIC!!!!!!!
                 */
                let Vec2 { x: a, y: b } = tilemap.tile_render_size;
                let c = c as f32;
                let Vec2 { x, y } = chunk_index.as_vec2();
                let n = tilemap.render_chunk_size as f32;

                let min = Vec2 {
                    x: a * x * n - a / 2. * y * n - (n / 2. - 1.) * a - a / 2.,
                    y: (b + c) / 2. * y * n,
                };
                let max = Vec2 {
                    x: a * x * n - a / 2. * y * n + 1. * a * n,
                    y: (b + c) / 2. * (y * n + n) - c / 2. + b / 2.,
                };

                tilemap.transform.transform_aabb(AabbBox2d { min, max })
            }
        }
    }

    pub fn from_tilemap_builder(builder: &TilemapBuilder) -> AabbBox2d {
        let pivot_offset = builder.pivot * builder.tile_slot_size;

        match builder.tile_type {
            TileType::Square => {
                let tilemap_render_size = builder.size.as_vec2() * builder.tile_slot_size;
                builder.transform.transform_aabb(AabbBox2d {
                    min: pivot_offset,
                    max: tilemap_render_size - pivot_offset,
                })
            }
            TileType::Isometric => {
                let half_size = builder.size.as_vec2() / 2.;
                let tilemap_render_size = (half_size.x + half_size.y) * builder.tile_slot_size;
                let center_x = (half_size.x - half_size.y) * builder.tile_slot_size.x / 2.;
                let center_y = tilemap_render_size.y / 2.;
                let center = Vec2 {
                    x: center_x,
                    y: center_y,
                } - pivot_offset;

                builder.transform.transform_aabb(AabbBox2d {
                    min: center - tilemap_render_size / 2.,
                    max: center + tilemap_render_size / 2.,
                })
            }
            TileType::Hexagonal(c) => {
                let c = c as f32;
                let Vec2 { x: m, y: n } = builder.size.as_vec2();
                let Vec2 { x: a, y: b } = builder.tile_slot_size;

                let min = Vec2 {
                    x: -(n / 2. - 0.5) * a,
                    y: 0.,
                } - pivot_offset;
                let max = Vec2 {
                    x: m * a,
                    y: (b + c) / 2. * n + (b - c) / 2.,
                } - pivot_offset;

                builder.transform.transform_aabb(AabbBox2d { min, max })
            }
        }
    }

    pub fn from_camera(camera: &ExtractedView) -> Self {
        let half_width = camera.width * camera.scale;
        let half_height = camera.height * camera.scale;
        AabbBox2d {
            min: Vec2 {
                x: camera.transform.x - half_width,
                y: camera.transform.y - half_height,
            },
            max: Vec2 {
                x: camera.transform.x + half_width,
                y: camera.transform.y + half_height,
            },
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
