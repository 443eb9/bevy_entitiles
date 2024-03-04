use std::ops::{Add, Div, Mul, Sub};

use bevy::{math::{IVec2, UVec2}, prelude::Vec2, reflect::Reflect, render::render_resource::ShaderType};

use crate::tilemap::map::{TilemapAxisFlip, TilemapTransform, TilemapType};

use super::{extension::Vec2Integerize, TileArea};

macro_rules! declare_aabb {
    ($aabb_ty: ident, $data_ty: ty) => {
        #[derive(Clone, Copy, Default, Debug, Reflect, ShaderType)]
        #[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
        pub struct $aabb_ty {
            pub min: $data_ty,
            pub max: $data_ty,
        }
    };
}

macro_rules! impl_aabb {
    ($aabb_ty: ty, $data_ty: ty, $acc_ty: ty) => {
        impl $aabb_ty {
            pub fn new(min_x: $acc_ty, min_y: $acc_ty, max_x: $acc_ty, max_y: $acc_ty) -> Self {
                Self {
                    min: <$data_ty>::new(min_x, min_y),
                    max: <$data_ty>::new(max_x, max_y),
                }
            }

            #[inline]
            pub fn splat(value: $data_ty) -> Self {
                Self {
                    min: value,
                    max: value,
                }
            }

            #[inline]
            pub fn width(&self) -> $acc_ty {
                self.max.x - self.min.x
            }

            #[inline]
            pub fn height(&self) -> $acc_ty {
                self.max.y - self.min.y
            }

            #[inline]
            pub fn top_left(&self) -> $data_ty {
                <$data_ty>::new(self.min.x, self.max.y)
            }

            #[inline]
            pub fn bottom_right(&self) -> $data_ty {
                <$data_ty>::new(self.max.x, self.min.y)
            }

            #[inline]
            pub fn area(&self) -> $acc_ty {
                self.width() * self.height()
            }

            #[inline]
            pub fn expand(&mut self, other: $aabb_ty) {
                self.min = self.min.min(other.min);
                self.max = self.max.max(other.max);
            }

            #[inline]
            pub fn expand_to_contain(&mut self, point: $data_ty) {
                self.min = self.min.min(point);
                self.max = self.max.max(point);
            }

            #[inline]
            pub fn is_intersected(&self, other: $aabb_ty) -> bool {
                self.min.x < other.max.x
                    && self.max.x > other.min.x
                    && self.min.y < other.max.y
                    && self.max.y > other.min.y
            }

            #[inline]
            pub fn contains(&self, point: $data_ty) -> bool {
                self.min.x <= point.x
                    && self.max.x >= point.x
                    && self.min.y <= point.y
                    && self.max.y >= point.y
            }

            #[inline]
            pub fn with_translation(&self, translation: $data_ty) -> Self {
                Self {
                    min: self.min + translation,
                    max: self.max + translation,
                }
            }

            #[inline]
            pub fn intersection(&self, other: $aabb_ty) -> $aabb_ty {
                Self {
                    min: self.min.max(other.min),
                    max: self.max.min(other.max),
                }
            }

            #[inline]
            pub fn is_subset_of(&self, other: $aabb_ty) -> bool {
                self.min.x >= other.min.x
                    && self.max.x <= other.max.x
                    && self.min.y >= other.min.y
                    && self.max.y <= other.max.y
            }

            #[inline]
            pub fn justify(&mut self) {
                if self.min.x > self.max.x {
                    std::mem::swap(&mut self.min.x, &mut self.max.x);
                }

                if self.min.y > self.max.y {
                    std::mem::swap(&mut self.min.y, &mut self.max.y);
                }
            }

            #[inline]
            pub fn justified(mut self) -> Self {
                self.justify();
                self
            }
        }

        impl From<[$data_ty; 2]> for $aabb_ty {
            fn from(value: [$data_ty; 2]) -> Self {
                Self {
                    min: value[0].into(),
                    max: value[1].into(),
                }
            }
        }

        impl Add<$data_ty> for $aabb_ty {
            type Output = Self;

            fn add(self, rhs: $data_ty) -> Self::Output {
                Self {
                    min: self.min + rhs,
                    max: self.max + rhs,
                }
            }
        }

        impl Sub<$data_ty> for $aabb_ty {
            type Output = Self;

            fn sub(self, rhs: $data_ty) -> Self::Output {
                Self {
                    min: self.min - rhs,
                    max: self.max - rhs,
                }
            }
        }

        impl Mul<$data_ty> for $aabb_ty {
            type Output = Self;

            fn mul(self, rhs: $data_ty) -> Self::Output {
                Self {
                    min: self.min * rhs,
                    max: self.max * rhs,
                }
            }
        }

        impl Div<$data_ty> for $aabb_ty {
            type Output = Self;

            fn div(self, rhs: $data_ty) -> Self::Output {
                Self {
                    min: self.min / rhs,
                    max: self.max / rhs,
                }
            }
        }
    };
}

declare_aabb!(Aabb2d, Vec2);
declare_aabb!(IAabb2d, IVec2);
declare_aabb!(UAabb2d, UVec2);

impl_aabb!(Aabb2d, Vec2, f32);
impl_aabb!(IAabb2d, IVec2, i32);
impl_aabb!(UAabb2d, UVec2, u32);

impl Aabb2d {
    pub fn from_tilemap(
        chunk_index: IVec2,
        chunk_size: u32,
        ty: TilemapType,
        tile_pivot: Vec2,
        axis_flip: TilemapAxisFlip,
        slot_size: Vec2,
        transform: TilemapTransform,
    ) -> Self {
        let pivot_offset = tile_pivot * slot_size;
        let chunk_index = chunk_index.as_vec2();
        let axis = axis_flip.as_vec2();
        let flipped = (axis - 1.) / 2.;
        let chunk_size = chunk_size as f32;

        transform.transform_aabb(
            match ty {
                TilemapType::Square => {
                    let chunk_render_size = slot_size * chunk_size;
                    Aabb2d {
                        min: chunk_index * chunk_render_size - pivot_offset,
                        max: (chunk_index + 1.) * chunk_render_size - pivot_offset,
                    } * axis
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

                    Aabb2d {
                        min: center - half_chunk_render_size,
                        max: center + half_chunk_render_size,
                    }
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

                    Aabb2d { min, max } - pivot_offset + flip_offset
                }
            }
            .justified(),
        )
    }

    #[inline]
    pub fn size(&self) -> Vec2 {
        self.max - self.min
    }

    #[inline]
    pub fn center(&self) -> Vec2 {
        (self.min + self.max) / 2.
    }

    #[inline]
    pub fn with_scale(&self, scale: Vec2, pivot: Vec2) -> Self {
        let size = self.size();
        let scaled_size = size * scale;
        let offset = (size - scaled_size) * pivot;

        Self {
            min: self.min + offset,
            max: self.max - offset,
        }
    }

    #[inline]
    pub fn expand_to_iaabb(&self) -> IAabb2d {
        IAabb2d {
            min: self.min.floor_to_ivec(),
            max: self.max.ceil_to_ivec(),
        }
    }

    #[inline]
    pub fn shrink_to_iaabb(&self) -> IAabb2d {
        IAabb2d {
            min: self.min.ceil_to_ivec(),
            max: self.max.floor_to_ivec(),
        }
    }
}

impl IAabb2d {
    #[inline]
    pub fn size(&self) -> IVec2 {
        self.max - self.min + 1
    }

    #[inline]
    pub fn into_iter(self) -> impl Iterator<Item = IVec2> {
        (self.min.y..=self.max.y)
            .into_iter()
            .map(move |y| {
                (self.min.x..=self.max.x)
                    .into_iter()
                    .map(move |x| IVec2 { x, y })
            })
            .flatten()
    }
}

impl Into<Aabb2d> for IAabb2d {
    fn into(self) -> Aabb2d {
        Aabb2d {
            min: self.min.as_vec2(),
            max: self.max.as_vec2(),
        }
    }
}

impl From<TileArea> for IAabb2d {
    fn from(value: TileArea) -> Self {
        Self {
            min: value.origin,
            max: value.dest,
        }
    }
}

impl UAabb2d {
    #[inline]
    pub fn size(&self) -> UVec2 {
        self.max - self.min + 1
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_aabb_scale() {
        let aabb = Aabb2d {
            min: Vec2::new(0., 0.),
            max: Vec2::new(10., 10.),
        };

        let scaled_aabb = aabb.with_scale(Vec2::new(2., 2.), Vec2::new(0.5, 0.5));

        assert_eq!(scaled_aabb.min, Vec2::new(-5., -5.));
        assert_eq!(scaled_aabb.max, Vec2::new(15., 15.));
    }
}
