use bevy::{
    app::{Plugin, Update},
    ecs::system::Resource,
    math::Vec2,
};

pub mod drawing;

pub struct EntiTilesDebugPlugin;

impl Plugin for EntiTilesDebugPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(
            Update,
            (
                drawing::draw_chunk_aabb,
                drawing::draw_tilemap_aabb,
                drawing::draw_axis,
                drawing::draw_camera_aabb,
                // #[cfg(feature = "algorithm")]
                // drawing::draw_path,
                #[cfg(feature = "serializing")]
                drawing::draw_updater_aabbs,
            ),
        );

        #[cfg(feature = "debug")]
        app.init_resource::<CameraAabbScale>();
    }
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct CameraAabbScale(pub Vec2);

impl Default for CameraAabbScale {
    fn default() -> Self {
        Self(Vec2::splat(1.))
    }
}
