use std::{fmt::Debug, time::Duration};

use bevy::{
    prelude::{default, Color, Commands, IntoSystemConfigs, Plugin, Startup, TextBundle, Update},
    text::{TextSection, TextStyle}, time::common_conditions::on_real_timer,
};

use crate::debug::drawing::{draw_chunk_aabb, draw_tilemap_aabb};

#[cfg(feature = "algorithm")]
use crate::debug::drawing::draw_path;

use self::common::{debug_info_display, DebugFpsText};

pub mod camera_movement;
pub mod common;
pub mod drawing;

/// A bunch of systems for debugging. Since they're not optimized, don't use them unless you're debugging.
pub struct EntiTilesDebugPlugin;

impl Plugin for EntiTilesDebugPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        println!("==============================");
        println!("Debug Enabled");
        println!("==============================");

        app.add_systems(Startup, debug_startup).add_systems(
            Update,
            (
                draw_tilemap_aabb,
                draw_chunk_aabb,
                #[cfg(feature = "algorithm")]
                draw_path,
                debug_info_display.run_if(on_real_timer(Duration::from_millis(100))),
            ),
        );

        app.add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin);
    }
}

pub fn debug_startup(mut commands: Commands) {
    commands.spawn((
        DebugFpsText,
        TextBundle::from_sections([
            TextSection::new(
                "FPS: ",
                TextStyle {
                    font_size: 32.,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            TextSection::new(
                "",
                TextStyle {
                    font_size: 32.,
                    color: Color::WHITE,
                    ..default()
                },
            ),
        ]),
    ));
}

pub fn validate_heap<K: PartialOrd + Debug, V: Debug>(tree: &Vec<(K, V)>, asc: bool) {
    for i in 1..tree.len() {
        if let Some(other) = tree.get(i * 2) {
            if asc {
                assert!(tree[i].0 <= other.0, "validate failed at {:?} <= {:?}", tree[i], other);
            } else {
                assert!(tree[i].0 >= other.0, "validate failed at {:?} >= {:?}", tree[i], other);
            }
        }

        if let Some(other) = tree.get(i * 2 + 1) {
            if asc {
                assert!(tree[i].0 <= other.0, "validate failed at {:?} <= {:?}", tree[i], other);
            } else {
                assert!(tree[i].0 >= other.0, "validate failed at {:?} >= {:?}", tree[i], other);
            }
        }
    }
    println!("heap validated √");
}
