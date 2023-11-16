use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::{Component, Query, Res, Resource, With, UVec2},
    text::Text, utils::HashMap,
};

#[cfg(feature = "algorithm")]
use crate::algorithm::pathfinding::{Path, PathNode};

#[derive(Component)]
pub struct DebugFpsText;

pub fn debug_info_display(
    mut query: Query<&mut Text, With<DebugFpsText>>,
    diag: Res<DiagnosticsStore>,
) {
    if let (Some(fps), Some(frame_time)) = (
        diag.get(FrameTimeDiagnosticsPlugin::FPS),
        diag.get(FrameTimeDiagnosticsPlugin::FRAME_TIME),
    ) {
        if let (Some(fps_value), Some(frame_time_value)) = (fps.smoothed(), frame_time.smoothed()) {
            let mut text = query.get_single_mut().unwrap();
            text.sections[1].value = format!("{fps_value:.2} ({frame_time_value:.2} ms)");
        }
    }
}
