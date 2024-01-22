use super::TiledColor;

pub(crate) fn default_onef() -> f32 {
    1.
}

pub(crate) fn default_true() -> bool {
    true
}

pub(crate) fn default_white() -> TiledColor {
    TiledColor {
        a: 1.,
        r: 1.,
        g: 1.,
        b: 1.,
    }
}
