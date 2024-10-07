use std::fmt::Display;

use bevy::{asset::AssetId, ecs::event::Event, math::Vec2, reflect::Reflect};

use crate::ldtk::{components::LevelIid, json::LdtkJson};

#[derive(Event, Clone)]
pub enum LdtkLevelEvent {
    Load(LdtkLevelLoader),
    Unload(LdtkLevelUnloader),
}

#[derive(Reflect, Default, Clone, Copy, PartialEq, Eq)]
pub enum LdtkLevelLoaderMode {
    #[default]
    Tilemap,
    MapPattern,
}

#[derive(Reflect, Clone)]
pub struct LdtkLevelLoader {
    pub json: AssetId<LdtkJson>,
    pub level: LdtkLevel,
    pub mode: LdtkLevelLoaderMode,
    /// Override the original tilemap translation or not.
    pub trans_ovrd: Option<Vec2>,
}

#[derive(Reflect, Clone)]
pub struct LdtkLevelUnloader {
    pub json: AssetId<LdtkJson>,
    pub level: LdtkLevel,
}

#[derive(Reflect, Debug, Clone)]
pub enum LdtkLevel {
    Identifier(String),
    Iid(LevelIid),
}

impl Display for LdtkLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LdtkLevel::Identifier(ident) => {
                f.write_fmt(format_args!("Ldtk level: Identifier {}", ident))
            }
            LdtkLevel::Iid(iid) => f.write_fmt(format_args!("Ldtk level: Iid {}", **iid)),
        }
    }
}
