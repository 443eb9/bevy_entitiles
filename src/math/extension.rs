use bevy::{
    math::IVec2,
    prelude::{UVec2, Vec2},
};

use crate::tilemap::map::TilemapType;

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
