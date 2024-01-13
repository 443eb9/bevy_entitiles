use bevy::{math::IVec2, prelude::Vec2, reflect::Reflect};

use crate::tilemap::map::{TilemapTransform, TilemapType};

use super::extension::Vec2Integerize;

#[derive(Clone, Copy, Default, Debug, Reflect)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub struct Aabb2d {
    pub min: Vec2,
    pub max: Vec2,
}

#[derive(Clone, Copy, Default, Debug, Reflect)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub struct IAabb2d {
    pub min: IVec2,
    pub max: IVec2,
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
        }
    };
}

impl_aabb!(Aabb2d, Vec2, f32);
impl_aabb!(IAabb2d, IVec2, i32);

impl Aabb2d {
    pub fn from_tilemap(
        chunk_index: IVec2,
        chunk_size: u32,
        ty: TilemapType,
        tile_pivot: Vec2,
        slot_size: Vec2,
        transform: TilemapTransform,
    ) -> Self {
        let pivot_offset = tile_pivot * slot_size;

        transform.transform_aabb(match ty {
            TilemapType::Square => {
                let chunk_render_size = slot_size * chunk_size as f32;
                Aabb2d {
                    min: chunk_index.as_vec2() * chunk_render_size - pivot_offset,
                    max: (chunk_index + 1).as_vec2() * chunk_render_size - pivot_offset,
                }
            }
            TilemapType::Isometric => {
                let chunk_index = chunk_index.as_vec2();
                let half_chunk_render_size = chunk_size as f32 * slot_size / 2.;
                let center_x = (chunk_index.x - chunk_index.y) * half_chunk_render_size.x;
                let center_y = (chunk_index.x + chunk_index.y + 1.) * half_chunk_render_size.y;
                let center = Vec2 {
                    x: center_x,
                    y: center_y,
                } - pivot_offset;

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
                let Vec2 { x, y } = chunk_index.as_vec2();
                let n = chunk_size as f32;

                let min = Vec2 {
                    x: a * x * n - a / 2. * y * n - (n / 2. - 1.) * a - a / 2.,
                    y: (b + c) / 2. * y * n,
                };
                let max = Vec2 {
                    x: a * x * n - a / 2. * y * n + 1. * a * n,
                    y: (b + c) / 2. * (y * n + n) - c / 2. + b / 2.,
                };

                Aabb2d { min, max }
            }
        })
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
        self.max - self.min + IVec2::ONE
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
