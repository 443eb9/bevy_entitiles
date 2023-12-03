use bevy::{
    asset::{Assets, Handle},
    ecs::{
        component::Component,
        entity::Entity,
        system::{Commands, Resource, Res},
        world::FromWorld,
    },
    math::{Vec2, UVec2},
    render::mesh::{shape, Mesh},
};

use crate::tilemap::{tile::{TileBuilder, TileType}, map::{Tilemap, TilemapBuilder}};

use super::EditorStyle;

#[derive(Component)]
pub struct PalatteTilemap;

#[derive(Resource, Default)]
pub struct TilePalatte {
    pub presets: Vec<TileBuilder>,
}

pub struct TilePreset {
    pub tile: TileBuilder,
    pub icon: Entity,
}

impl TilePreset {
    pub fn new(commands: &mut Commands, tile: &TileBuilder) {}
}
