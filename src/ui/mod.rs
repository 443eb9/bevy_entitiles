use bevy::{
    app::{Plugin, Update},
    asset::{load_internal_asset, Handle},
    render::{render_resource::Shader, ExtractSchedule, RenderApp},
};

use crate::{tilemap::ui::update_index, ui::render::extract};

pub mod pipeline;
pub mod render;
pub mod resources;
pub mod uniform;

pub const UI_TILES_SHADER: Handle<Shader> = Handle::weak_from_u128(2135135645316312);

pub struct EntiTilesUiPlugin;

impl Plugin for EntiTilesUiPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        load_internal_asset!(app, UI_TILES_SHADER, "ui_tiles.wgsl", Shader::from_wgsl);

        app.add_systems(Update, update_index);

        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_systems(ExtractSchedule, extract);
    }
}
