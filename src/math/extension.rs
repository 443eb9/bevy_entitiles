use bevy::prelude::{UVec2, Vec2};

pub trait F32ToU32 {
    fn round_to_u32(self) -> u32;
    fn ceil_to_u32(self) -> u32;
    fn floor_to_u32(self) -> u32;
}

impl F32ToU32 for f32 {
    fn round_to_u32(self) -> u32 {
        if self.fract() > 0.5 {
            self as u32 + 1
        } else {
            self as u32
        }
    }

    fn ceil_to_u32(self) -> u32 {
        self as u32 + 1
    }

    fn floor_to_u32(self) -> u32 {
        self as u32
    }
}

pub trait Vec2ToUVec2 {
    fn round_to_u32(self) -> UVec2;
    fn ceil_to_u32(self) -> UVec2;
}

impl Vec2ToUVec2 for Vec2 {
    fn round_to_u32(self) -> UVec2 {
        UVec2 {
            x: self.x.round_to_u32(),
            y: self.y.round_to_u32(),
        }
    }

    fn ceil_to_u32(self) -> UVec2 {
        UVec2 {
            x: self.x.ceil_to_u32(),
            y: self.y.ceil_to_u32(),
        }
    }
}
