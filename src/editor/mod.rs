use bevy::{
    app::{Plugin, Startup},
    asset::{Asset, Assets, Handle},
    ecs::{
        entity::Entity,
        schedule::{OnEnter, States},
        system::{Commands, ResMut, Resource},
        world::FromWorld,
    },
    math::Vec2,
    render::mesh::{shape, Mesh},
};

use crate::{tilemap::tile::TileBuilder, EntiTilesStates};

use self::{palatte::TilePalatte, ui::setup_ui};

pub mod palatte;
pub mod ui;

pub struct EntiTilesEditorPlugin;

impl Plugin for EntiTilesEditorPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, setup_ui)
            .add_systems(OnEnter(EntiTilesStates::Editing), setup_ui);

        app.init_resource::<TilePalatte>()
            .init_resource::<EditorStyle>();
    }
}

#[derive(Resource)]
pub struct EditorStyle {
    pub title_size: f32,
    pub max_presets: usize,
    pub preset_icon_size: Vec2,
    pub preset_margin: f32,
}

impl Default for EditorStyle {
    fn default() -> Self {
        Self {
            title_size: 40.,
            max_presets: 300,
            preset_icon_size: Vec2 { x: 30., y: 30. },
            preset_margin: 10.,
        }
    }
}
