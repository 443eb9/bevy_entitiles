use std::{fmt::Debug, time::Duration};

use bevy::{
    prelude::{default, Color, Commands, IntoSystemConfigs, Plugin, Startup, TextBundle, Update},
    text::{TextSection, TextStyle},
    time::common_conditions::on_real_timer,
};

use drawing::{draw_chunk_aabb, draw_tilemap_aabb};

use crate::helpers::camera_movement::camera_control;

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
                camera_control,
                #[cfg(feature = "algorithm")]
                drawing::draw_path,
                debug_info_display.run_if(on_real_timer(Duration::from_millis(100))),
            ),
        );

        app.add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin);
    }

    fn finish(&self, _app: &mut bevy::prelude::App) {
        // print_render_graph(_app);
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

pub fn validate_heap<K: PartialOrd + Debug, V: Debug>(tree: &Vec<Option<(K, V)>>, asc: bool) {
    for i in 1..tree.len() {
        if let Some((k1, _)) = &tree[i] {
            let left = i * 2;
            let right = i * 2 + 1;
            if left < tree.len() {
                if let Some((k2, _)) = &tree[left] {
                    if asc {
                        assert!(k1 <= k2, "heap validation failed at index {}", i);
                    } else {
                        assert!(k1 >= k2, "heap validation failed at index {}", i);
                    }
                }
            }
            if right < tree.len() {
                if let Some((k2, _)) = &tree[right] {
                    if asc {
                        assert!(k1 <= k2, "heap validation failed at index {}", i);
                    } else {
                        assert!(k1 >= k2, "heap validation failed at index {}", i);
                    }
                }
            }
        }
    }
    println!("heap validated √");
}
