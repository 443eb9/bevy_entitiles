use bevy::{
    math::IVec2,
    prelude::{UVec2, Vec2},
};

use crate::tilemap::tile::TileType;

pub trait F32ToU32 {
    fn round_to_u32(self) -> u32;
    fn ceil_to_u32(self) -> u32;
    fn floor_to_u32(self) -> u32;
}

impl F32ToU32 for f32 {
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

pub trait Vec2ToUVec2 {
    fn round_to_uvec(self) -> UVec2;
    fn ceil_to_uvec(self) -> UVec2;
    fn floor_to_uvec(self) -> UVec2;
    fn ceil_x_floor_y_to_uvec(self) -> UVec2;
    fn floor_x_ceil_y_to_uvec(self) -> UVec2;
}

impl Vec2ToUVec2 for Vec2 {
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

    #[inline]
    fn ceil_x_floor_y_to_uvec(self) -> UVec2 {
        UVec2 {
            x: self.x.ceil_to_u32(),
            y: self.y.floor_to_u32(),
        }
    }

    #[inline]
    fn floor_x_ceil_y_to_uvec(self) -> UVec2 {
        UVec2 {
            x: self.x.floor_to_u32(),
            y: self.y.ceil_to_u32(),
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

pub trait TileIndex<T> {
    fn neighbours(self, ty: TileType, allow_diagonal: bool) -> Vec<T>;
}

impl TileIndex<UVec2> for UVec2 {
    fn neighbours(self, ty: TileType, allow_diagonal: bool) -> Vec<UVec2> {
        let mut result = Vec::with_capacity(8);
        match ty {
            TileType::Hexagonal(_) => {
                for d in [
                    IVec2::ONE,
                    IVec2::ONE,
                    IVec2::NEG_ONE,
                    IVec2::NEG_ONE,
                    IVec2::X,
                    IVec2::Y,
                ] {
                    let index = IVec2 {
                        x: (self.x as i32 + d.x),
                        y: (self.y as i32 + d.y),
                    };
                    if index.x >= 0 || index.y >= 0 {
                        result.push(index.as_uvec2());
                    }
                }
            }
            _ => {
                for dy in [-1, 0, 1] {
                    for dx in [-1, 0, 1] {
                        if dx == 0 && dy == 0 {
                            continue;
                        }

                        if !allow_diagonal && dx != 0 && dy != 0 {
                            continue;
                        }

                        let index = IVec2 {
                            x: (self.x as i32 + dx),
                            y: (self.y as i32 + dy),
                        };
                        if index.x >= 0 || index.y >= 0 {
                            result.push(index.as_uvec2());
                        }
                    }
                }
            }
        }
        result
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
