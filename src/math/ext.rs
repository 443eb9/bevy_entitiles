use bevy::{
    math::{IRect, IVec2, Rect, URect},
    prelude::{UVec2, Vec2},
};

use crate::tilemap::map::{TilemapAxisFlip, TilemapTransform, TilemapType};

pub trait F32Integerize {
    fn round_to_i32(self) -> i32;
    fn ceil_to_i32(self) -> i32;
    fn floor_to_i32(self) -> i32;
    fn round_to_u32(self) -> u32;
    fn ceil_to_u32(self) -> u32;
    fn floor_to_u32(self) -> u32;
}

impl F32Integerize for f32 {
    #[inline]
    fn round_to_i32(self) -> i32 {
        self.round() as i32
    }

    #[inline]
    fn ceil_to_i32(self) -> i32 {
        self.ceil() as i32
    }

    #[inline]
    fn floor_to_i32(self) -> i32 {
        self.floor() as i32
    }

    #[inline]
    fn round_to_u32(self) -> u32 {
        (self + 0.5) as u32
    }

    #[inline]
    fn ceil_to_u32(self) -> u32 {
        self as u32 + 1
    }

    #[inline]
    fn floor_to_u32(self) -> u32 {
        self as u32
    }
}

pub trait Vec2Integerize {
    fn round_to_ivec(self) -> IVec2;
    fn ceil_to_ivec(self) -> IVec2;
    fn floor_to_ivec(self) -> IVec2;
    fn round_to_uvec(self) -> UVec2;
    fn ceil_to_uvec(self) -> UVec2;
    fn floor_to_uvec(self) -> UVec2;
}

impl Vec2Integerize for Vec2 {
    #[inline]
    fn round_to_ivec(self) -> IVec2 {
        self.round().as_ivec2()
    }

    #[inline]
    fn ceil_to_ivec(self) -> IVec2 {
        self.ceil().as_ivec2()
    }

    #[inline]
    fn floor_to_ivec(self) -> IVec2 {
        self.floor().as_ivec2()
    }

    #[inline]
    fn round_to_uvec(self) -> UVec2 {
        UVec2 {
            x: self.x.round_to_u32(),
            y: self.y.round_to_u32(),
        }
    }

    #[inline]
    fn ceil_to_uvec(self) -> UVec2 {
        UVec2 {
            x: self.x.ceil_to_u32(),
            y: self.y.ceil_to_u32(),
        }
    }

    #[inline]
    fn floor_to_uvec(self) -> UVec2 {
        UVec2 {
            x: self.x as u32,
            y: self.y as u32,
        }
    }
}

pub trait ManhattanDistance<T> {
    fn manhattan_distance(self, other: Self) -> T;
}

impl ManhattanDistance<u32> for UVec2 {
    fn manhattan_distance(self, other: Self) -> u32 {
        let d = (self.as_ivec2() - other.as_ivec2()).abs().as_uvec2();
        d.x + d.y
    }
}

impl ManhattanDistance<u32> for IVec2 {
    fn manhattan_distance(self, other: Self) -> u32 {
        let d = (self - other).abs();
        d.x as u32 + d.y as u32
    }
}

pub trait TileIndex<T> {
    fn neighbours(self, ty: TilemapType, allow_diagonal: bool) -> Vec<Option<T>>;
}

impl TileIndex<IVec2> for IVec2 {
    fn neighbours(self, ty: TilemapType, allow_diagonal: bool) -> Vec<Option<IVec2>> {
        match ty {
            TilemapType::Hexagonal(_) => [
                IVec2::ONE,
                IVec2::ONE,
                IVec2::NEG_ONE,
                IVec2::NEG_ONE,
                IVec2::X,
                IVec2::Y,
            ]
            .into_iter()
            .map(|p| Some(p + self))
            .collect(),
            _ => {
                let seq = [
                    IVec2::Y,
                    IVec2::X,
                    IVec2::NEG_X,
                    IVec2::NEG_Y,
                    IVec2::ONE,
                    IVec2::NEG_ONE,
                    IVec2::new(1, -1),
                    IVec2::new(-1, 1),
                ]
                .into_iter()
                .map(|p| Some(p + self))
                .collect();
                if allow_diagonal {
                    seq
                } else {
                    seq[0..4].to_vec()
                }
            }
        }
    }
}

impl TileIndex<UVec2> for UVec2 {
    fn neighbours(self, ty: TilemapType, allow_diagonal: bool) -> Vec<Option<UVec2>> {
        match ty {
            TilemapType::Hexagonal(_) => [
                IVec2::ONE,
                IVec2::ONE,
                IVec2::NEG_ONE,
                IVec2::NEG_ONE,
                IVec2::X,
                IVec2::Y,
            ]
            .into_iter()
            .map(|p| {
                let nei = p + self.as_ivec2();
                if nei.x >= 0 && nei.y >= 0 {
                    Some(nei.as_uvec2())
                } else {
                    None
                }
            })
            .collect(),
            _ => {
                let seq = [
                    IVec2::Y,
                    IVec2::X,
                    IVec2::NEG_X,
                    IVec2::NEG_Y,
                    IVec2::ONE,
                    IVec2::NEG_ONE,
                    IVec2::new(1, -1),
                    IVec2::new(-1, 1),
                ]
                .into_iter()
                .map(|p| {
                    let nei = p + self.as_ivec2();
                    if nei.x >= 0 && nei.y >= 0 {
                        Some(nei.as_uvec2())
                    } else {
                        None
                    }
                })
                .collect();
                if allow_diagonal {
                    seq
                } else {
                    seq[0..4].to_vec()
                }
            }
        }
    }
}

pub trait DivToCeil {
    fn div_to_ceil(self, other: Self) -> Self;
}

impl DivToCeil for IVec2 {
    fn div_to_ceil(self, other: Self) -> Self {
        let mut result = self / other;
        if self.x % other.x != 0 && self.x > 0 {
            result.x += 1;
        }
        if self.y % other.y != 0 && self.x > 0 {
            result.y += 1;
        }
        result
    }
}

impl DivToCeil for UVec2 {
    fn div_to_ceil(self, other: Self) -> Self {
        let mut result = self / other;
        if self.x % other.x != 0 {
            result.x += 1;
        }
        if self.y % other.y != 0 {
            result.y += 1;
        }
        result
    }
}

pub trait DivToFloor {
    fn div_to_floor(self, other: Self) -> Self;
}

impl DivToFloor for IVec2 {
    fn div_to_floor(self, other: Self) -> Self {
        let mut result = self / other;
        if self.x % other.x != 0 && self.x < 0 {
            result.x -= 1;
        }
        if self.y % other.y != 0 && self.y < 0 {
            result.y -= 1;
        }
        result
    }
}

impl DivToFloor for UVec2 {
    fn div_to_floor(self, other: Self) -> Self {
        let mut result = self / other;
        if self.x % other.x != 0 {
            result.x -= 1;
        }
        if self.y % other.y != 0 {
            result.y -= 1;
        }
        result
    }
}

pub trait ChunkIndex {
    fn chunk_file_name(self) -> String;
}

impl ChunkIndex for IVec2 {
    fn chunk_file_name(self) -> String {
        format!("{}_{}", self.x, self.y)
    }
}

pub trait RectFromTilemap {
    fn from_tilemap(
        chunk_index: IVec2,
        chunk_size: u32,
        ty: TilemapType,
        tile_pivot: Vec2,
        axis_flip: TilemapAxisFlip,
        slot_size: Vec2,
        transform: TilemapTransform,
    ) -> Rect;
}

impl RectFromTilemap for Rect {
    fn from_tilemap(
        chunk_index: IVec2,
        chunk_size: u32,
        ty: TilemapType,
        tile_pivot: Vec2,
        axis_flip: TilemapAxisFlip,
        slot_size: Vec2,
        transform: TilemapTransform,
    ) -> Rect {
        let pivot_offset = tile_pivot * slot_size;
        let chunk_index = chunk_index.as_vec2();
        let axis = axis_flip.as_vec2();
        let flipped = (axis - 1.) / 2.;
        let chunk_size = chunk_size as f32;

        transform.transform_rect(match ty {
            TilemapType::Square => {
                let chunk_render_size = slot_size * chunk_size;
                Rect::from_corners(
                    (chunk_index * chunk_render_size - pivot_offset) * axis,
                    ((chunk_index + 1.) * chunk_render_size - pivot_offset) * axis,
                )
            }
            TilemapType::Isometric => {
                let chunk_index = chunk_index * axis;
                let half_chunk_render_size = chunk_size * slot_size / 2.;
                let center_x = (chunk_index.x - chunk_index.y) * half_chunk_render_size.x;
                let center_y = (chunk_index.x + chunk_index.y + 1.) * half_chunk_render_size.y;

                let flip_offset = Vec2 {
                    x: if axis_flip.is_all() {
                        0.5
                    } else {
                        flipped.x * (chunk_size / 2. - 1.) - flipped.y * chunk_size / 2.
                    },
                    y: (flipped.x + flipped.y) * chunk_size / 2.,
                } * slot_size;

                let center = Vec2 {
                    x: center_x,
                    y: center_y,
                } - pivot_offset
                    + flip_offset;

                Rect::from_corners(
                    center - half_chunk_render_size,
                    center + half_chunk_render_size,
                )
            }
            TilemapType::Hexagonal(c) => {
                /*
                 * MATHEMATICAL MAGIC!!!!!!!
                 */
                let Vec2 { x: a, y: b } = slot_size;
                let c = c as f32;
                let Vec2 { x, y } = chunk_index * axis;
                let n = chunk_size;

                let min = Vec2 {
                    x: a * n * (x - y / 2.) - (n / 2. - 0.5) * a,
                    y: (b + c) / 2. * y * n,
                };
                let max = Vec2 {
                    x: a * n * (x - y / 2. + 1.),
                    y: (b + c) / 2. * (y * n + n) - c / 2. + b / 2.,
                };

                let flip_offset = Vec2 {
                    x: ((flipped.x - flipped.y / 2.) * (chunk_size - 1.)) * a,
                    y: (b + c) / 2. * flipped.y * (chunk_size - 1.),
                } - (1. - axis) / 2. * slot_size;

                Rect::from_corners(
                    min - pivot_offset + flip_offset,
                    max - pivot_offset + flip_offset,
                )
            }
        })
    }
}

pub trait RectTransformation<T> {
    fn with_translation(self, x: T) -> Self;
    fn translate(&mut self, x: T);
    fn with_scale(self, x: T, pivot: T) -> Self;
    fn scale(&mut self, x: T, pivot: T);
}

macro_rules! impl_rect_transformation {
    ($rect_ty: ident, $data_ty: ident, $precision: ident) => {
        impl RectTransformation<$data_ty> for $rect_ty {
            #[inline]
            fn with_translation(mut self, x: $data_ty) -> Self {
                self.translate(x);
                self
            }

            #[inline]
            fn translate(&mut self, x: $data_ty) {
                self.min += x;
                self.max += x;
            }

            #[inline]
            fn with_scale(mut self, x: $data_ty, pivot: $data_ty) -> Self {
                self.scale(x, pivot);
                self
            }

            #[inline]
            fn scale(&mut self, x: $data_ty, pivot: $data_ty) {
                let offset = pivot * (($data_ty::ONE - x) / (2 as $precision) * self.size());
                self.min += offset;
                self.max -= offset;
            }
        }
    };
}

impl_rect_transformation!(Rect, Vec2, f32);
impl_rect_transformation!(IRect, IVec2, i32);
impl_rect_transformation!(URect, UVec2, u32);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_v2u() {
        assert_eq!(Vec2::new(1.5, 1.5).round_to_uvec(), UVec2::new(2, 2));
        assert_eq!(Vec2::new(1.4, 1.4).round_to_uvec(), UVec2::new(1, 1));
        assert_eq!(Vec2::new(1.6, 1.6).round_to_uvec(), UVec2::new(2, 2));
        assert_eq!(Vec2::new(1.9, 1.9).round_to_uvec(), UVec2::new(2, 2));

        assert_eq!(Vec2::new(1.5, 1.5).ceil_to_uvec(), UVec2::new(2, 2));
        assert_eq!(Vec2::new(1.4, 1.4).ceil_to_uvec(), UVec2::new(2, 2));
        assert_eq!(Vec2::new(1.6, 1.6).ceil_to_uvec(), UVec2::new(2, 2));
        assert_eq!(Vec2::new(1.9, 1.9).ceil_to_uvec(), UVec2::new(2, 2));

        println!("passed");
    }
}
