use bevy::{
    math::Vec3Swizzles,
    prelude::{UVec2, Vec2},
};

use crate::{
    render::extract::{ExtractedTilemap, ExtractedView},
    tilemap::{TileType, TilemapBuilder},
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
                    min: chunk_index.as_vec2() * chunk_render_size
                        + tilemap.transfrom.translation.xy(),
                    max: (chunk_index + 1).as_vec2() * chunk_render_size
                        + tilemap.transfrom.translation.xy(),
                }
            }
            TileType::IsometricDiamond => {
                let chunk_index = chunk_index.as_vec2();
                let chunk_render_size = tilemap.render_chunk_size as f32 * tilemap.tile_render_size;
                let center_x = tilemap.tile_render_size.x / 2.
                    + (chunk_index.x - chunk_index.y) / 2. * chunk_render_size.x;
                let center_y = chunk_render_size.y / 2.
                    + (chunk_index.x + chunk_index.y) / 2. * chunk_render_size.y;

                AabbBox2d {
                    min: Vec2::new(
                        center_x - chunk_render_size.x / 2. + tilemap.transfrom.translation.x,
                        center_y - chunk_render_size.y / 2. + tilemap.transfrom.translation.y,
                    ),
                    max: Vec2::new(
                        center_x + chunk_render_size.x / 2. + tilemap.transfrom.translation.x,
                        center_y + chunk_render_size.y / 2. + tilemap.transfrom.translation.y,
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
                    min: (tilemap_render_size - tilemap_render_size) / 2. + builder.transform,
                    max: (tilemap_render_size + tilemap_render_size) / 2. + builder.transform,
                }
            }
            TileType::IsometricDiamond => {
                let size = builder.size.as_vec2();
                let tilemap_render_size = (size.x + size.y) / 2. * builder.tile_render_size;
                let center_x = (size.x - size.y + 2.) * builder.tile_render_size.x / 4.;
                let center_y = tilemap_render_size.y / 2.;
                AabbBox2d {
                    min: Vec2::new(
                        center_x - tilemap_render_size.x / 2. + builder.transform.x,
                        center_y - tilemap_render_size.y / 2. + builder.transform.y,
                    ),
                    max: Vec2::new(
                        center_x + tilemap_render_size.x / 2. + builder.transform.x,
                        center_y + tilemap_render_size.y / 2. + builder.transform.y,
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

    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }

    pub fn height(&self) -> f32 {
        self.max.y - self.min.y
    }

    pub fn center(&self) -> Vec2 {
        (self.min + self.max) / 2.
    }

    pub fn area(&self) -> f32 {
        self.width() * self.height()
    }

    pub fn expand(&mut self, other: &AabbBox2d) {
        self.min = self.min.min(other.min);
        self.max = self.max.max(other.max);
    }

    pub fn is_intersected_with(&self, other: &AabbBox2d) -> bool {
        self.min.x < other.max.x
            && self.max.x > other.min.x
            && self.min.y < other.max.y
            && self.max.y > other.min.y
    }
}
