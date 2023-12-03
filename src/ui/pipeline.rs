use bevy::{
    ecs::{system::Resource, world::FromWorld},
    render::render_resource::BindGroupLayout,
};

use crate::render::resources::TilemapBindGroupLayouts;

#[derive(Resource)]
pub struct EntiTilesUiPipeline {
    pub ui_tilemap_uniform_layout: BindGroupLayout,
    pub colored_texture_layout: BindGroupLayout,
}

impl FromWorld for EntiTilesUiPipeline {
    fn from_world(world: &mut bevy::prelude::World) -> Self {
        let layouts = world.resource::<TilemapBindGroupLayouts>();
        Self {
            ui_tilemap_uniform_layout: layouts.ui_tilemap_uniform_layout.clone(),
            colored_texture_layout: layouts.color_texture_layout.clone(),
        }
    }
}
