use std::{fs::read_to_string, io::Read, path::Path};

use bevy::{
    prelude::{App, Camera2dBundle, Commands, Startup, UVec2, Update, Vec2},
    DefaultPlugins,
};
use bevy_entitiles::{
    algorithm::wfc::{WaveFunctionCollapser, WfcMode},
    debug::camera_movement::camera_control,
    math::FillArea,
    tilemap::{TileType, TilemapBuilder},
    EntiTilesPlugin,
};
use ron::de;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EntiTilesPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, camera_control)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    let (tilemap_entity, tilemap) = TilemapBuilder::new(
        TileType::Square,
        UVec2 { x: 5, y: 5 },
        Vec2 { x: 32., y: 32. },
    )
    .build(&mut commands);

    commands
        .entity(tilemap_entity)
        .insert(WaveFunctionCollapser::from_config(
            "examples/wfc_config.ron".into(),
            WfcMode::NonWeighted,
            FillArea::full(&tilemap),
            None,
            Some(1),
        ));
}
