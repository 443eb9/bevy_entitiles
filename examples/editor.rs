use bevy::{
    app::{App, Startup, Update},
    core_pipeline::core_2d::Camera2dBundle,
    ecs::{
        schedule::{NextState, OnEnter},
        system::{Commands, Res, ResMut},
    },
    input::{keyboard::KeyCode, Input},
    DefaultPlugins,
};
use bevy_entitiles::{editor::EntiTilesEditorPlugin, EntiTilesPlugin, EntiTilesStates};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EntiTilesPlugin, EntiTilesEditorPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, toggle_editor)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn toggle_editor(mut next_state: ResMut<NextState<EntiTilesStates>>, input: Res<Input<KeyCode>>) {
    if input.just_pressed(KeyCode::AltLeft) {
        next_state.set(EntiTilesStates::Simulating);
    }
    if input.just_pressed(KeyCode::AltRight) {
        next_state.set(EntiTilesStates::Editing);
    }
}
