use bevy::prelude::{UVec2, Vec2};

pub trait F32ToU32 {
    fn round_to_u32(self) -> u32;
    fn ceil_to_u32(self) -> u32;
    fn floor_to_u32(self) -> u32;
}

impl F32ToU32 for f32 {
    #[inline]
    fn round_to_u32(self) -> u32 {
        if self.fract() >= 0.5 {
            self as u32 + 1
        } else {
            self as u32
        }
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
