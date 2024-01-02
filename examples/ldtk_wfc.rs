use bevy::{
    app::{App, Startup},
    core_pipeline::core_2d::Camera2dBundle,
    ecs::system::{Commands, Res, ResMut},
    math::UVec2,
    DefaultPlugins,
};
use bevy_entitiles::{
    algorithm::wfc::{WfcRules, WfcRunner, WfcSource},
    ldtk::resources::{LdtkLevelManager, LdtkLevelManagerMode, LdtkPatterns},
    math::TileArea,
    tilemap::tile::TileType,
    EntiTilesPlugin,
};
use helpers::EntiTilesDebugPlugin;

mod helpers;

fn main() {
    let mut app = App::new();
    app.add_plugins((DefaultPlugins, EntiTilesPlugin, EntiTilesDebugPlugin))
        .add_systems(Startup, setup)
        .insert_resource(LdtkLevelManager::new(
            "assets/ldtk/wfc_source.ldtk",
            "ldtk/",
            LdtkLevelManagerMode::MapPattern,
        ));
    let id = app.world.register_system(load_map);
    app.insert_resource(LdtkPatterns {
        callback: Some(id),
        threashold: Some(6),
        ..Default::default()
    })
    .run();
}

fn setup(mut commands: Commands, mut manager: ResMut<LdtkLevelManager>) {
    commands.spawn(Camera2dBundle::default());

    manager.load_all(&mut commands);
}

fn load_map(mut commands: Commands, patterns: Res<LdtkPatterns>) {
    let mut map_source = vec![None; patterns.patterns.len()];

    patterns.patterns.iter().for_each(|(ident, p)| {
        map_source[ident.split("_").last().unwrap().parse::<usize>().unwrap()] =
            Some((p.0.clone(), Some(p.1.clone())));
    });

    let rules = WfcRules::from_file("examples/ldtk_wfc_config.ron", TileType::Square);
    commands.spawn((
        WfcRunner::new(
            TileType::Square,
            rules,
            TileArea::new_unchecked(UVec2::ZERO, UVec2 { x: 4, y: 4 }),
            None,
        ),
        WfcSource::MultiLayerMapPattern(
            UVec2::splat(16),
            map_source.into_iter().map(|p| p.unwrap()).collect(),
        ),
    ));
}
