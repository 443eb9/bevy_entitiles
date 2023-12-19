use bevy::{reflect::Reflect, render::render_resource::FilterMode};

#[derive(Reflect, Debug, PartialEq, Eq, Clone, Copy, Default)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub enum ReflectFilterMode {
    #[default]
    Nearest = 0,
    Linear = 1,
}

impl From<FilterMode> for ReflectFilterMode {
    fn from(value: FilterMode) -> Self {
        match value {
            FilterMode::Nearest => Self::Nearest,
            FilterMode::Linear => Self::Linear,
        }
    }
}

impl Into<FilterMode> for ReflectFilterMode {
    fn into(self) -> FilterMode {
        match self {
            Self::Nearest => FilterMode::Nearest,
            Self::Linear => FilterMode::Linear,
        }
    }
}
