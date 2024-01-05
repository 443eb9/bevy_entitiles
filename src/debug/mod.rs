use bevy::app::{Plugin, Update};

pub mod drawing;

pub struct EntiTilesDebugPlugin;

impl Plugin for EntiTilesDebugPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(
            Update,
            (
                drawing::draw_chunk_aabb,
                drawing::draw_axis,
                #[cfg(feature = "algorithm")]
                drawing::draw_path,
            ),
        );
    }
}
