use std::time::Duration;

use bevy::{
    prelude::{default, Color, Commands, IntoSystemConfigs, Plugin, Startup, TextBundle, Update},
    text::{TextSection, TextStyle},
    time::common_conditions::on_real_timer,
};

use crate::helpers::camera_movement::camera_control;

use self::{
    camera_movement::CameraControl,
    common::{debug_info_display, DebugFpsText},
};

pub mod camera_movement;
pub mod common;

pub struct EntiTilesHelpersPlugin {
    #[allow(unused)]
    pub inspector: bool,
}

impl Default for EntiTilesHelpersPlugin {
    fn default() -> Self {
        Self { inspector: true }
    }
}

impl Plugin for EntiTilesHelpersPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, debug_startup).add_systems(
            Update,
            (
                camera_control,
                debug_info_display.run_if(on_real_timer(Duration::from_millis(100))),
            ),
        );

        app.add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin);

        #[cfg(not(target_arch = "wasm32"))]
        if self.inspector {
            app.add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::default());
        }

        app.init_resource::<CameraControl>();
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
