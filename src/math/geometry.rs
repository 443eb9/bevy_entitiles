use bevy::prelude::{UVec2, Vec2};

use crate::{
    render::extract::ExtractedTilemap,
    tilemap::{TileType, TilemapBuilder},
};

#[derive(Clone, Default)]
pub struct AabbBox2d {
    pub center: Vec2,
    pub width: f32,
    pub height: f32,
}

impl AabbBox2d {
    pub fn from_chunk(chunk_index: UVec2, tilemap: &ExtractedTilemap) -> Self {
        match tilemap.tile_type {
            TileType::Square => {
                let chunk_render_size = tilemap.size.as_vec2() * tilemap.tile_render_size;
                AabbBox2d {
                    center: chunk_index.as_vec2() * tilemap.tile_render_size
                        + chunk_render_size / 2.,
                    width: chunk_render_size.x,
                    height: chunk_render_size.y,
                }
            }
            TileType::IsometricDiamond => {
                let chunk_index = chunk_index.as_vec2();
                let chunk_render_size = tilemap.render_chunk_size as f32 * tilemap.tile_render_size;
                
                AabbBox2d {
                    center: Vec2::new(tilemap.tile_render_size.x / 2., chunk_render_size.y / 2.)
                        + Vec2::new(
                            (chunk_index.x - chunk_index.y) / 2. * chunk_render_size.x,
                            (chunk_index.x + chunk_index.y) / 2. * chunk_render_size.y,
                        ),
                    width: chunk_render_size.x,
                    height: chunk_render_size.y,
                }
            }
        }
    }

    pub fn from_tilemap_builder(builder: &TilemapBuilder) -> AabbBox2d {
        match builder.tile_type {
            TileType::Square => {
                let tilemap_render_size = builder.size.as_vec2() * builder.tile_render_size;
                AabbBox2d {
                    center: tilemap_render_size / 2.,
                    width: tilemap_render_size.x,
                    height: tilemap_render_size.y,
                }
            }
            TileType::IsometricDiamond => {
                let t = (builder.size.x + builder.size.y) as f32 / 2.;
                let tilemap_render_size = t * builder.tile_render_size;
                AabbBox2d {
                    center: Vec2::new(builder.tile_render_size.x / 2., tilemap_render_size.y / 2.)
                        + Vec2::new(
                            (builder.size.x as f32 - builder.size.y as f32)
                                * builder.tile_render_size.x
                                / 4.,
                            0.,
                        ),
                    width: tilemap_render_size.x,
                    height: tilemap_render_size.y,
                }
            }
        }
    }

    pub fn is_intersected_with(&self, lhs_offset: Vec2, rhs: &AabbBox2d, rhs_offset: Vec2) -> bool {
        let mut l = self.clone();
        let mut r = rhs.clone();

        l.center += lhs_offset;
        r.center += rhs_offset;

        (l.center.x - r.center.x).abs() * 2. < (l.width + r.width)
            && (l.center.y - r.center.y).abs() * 2. < (l.height + r.height)
    }
}
