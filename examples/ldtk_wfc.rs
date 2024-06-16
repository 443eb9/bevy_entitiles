use bevy::{
    app::{App, Startup, Update},
    color::Color,
    core_pipeline::core_2d::Camera2dBundle,
    ecs::{
        component::Component,
        entity::Entity,
        system::{Commands, Query, Res, ResMut},
    },
    gizmos::{config::GizmoConfig, AppGizmoBuilder},
    input::{keyboard::KeyCode, ButtonInput},
    math::{IVec2, UVec2, Vec2, Vec3Swizzles},
    reflect::Reflect,
    render::render_resource::FilterMode,
    sprite::{Sprite, SpriteBundle},
    transform::components::Transform,
    utils::HashMap,
    DefaultPlugins,
};
use bevy_entitiles::{
    algorithm::wfc::{LdtkWfcMode, WfcRules, WfcRunner, WfcSource},
    ldtk::{
        layer::physics::LdtkPhysicsLayer,
        resources::{
            LdtkAdditionalLayers, LdtkLevelManager, LdtkLoadConfig, LdtkPatterns, LdtkWfcManager,
        },
    },
    math::TileArea,
    tilemap::{map::TilemapType, physics::PhysicsTile},
    EntiTilesPlugin,
};
use bevy_xpbd_2d::plugins::{debug::PhysicsGizmos, PhysicsDebugPlugin, PhysicsPlugins};
use helpers::EntiTilesHelpersPlugin;

mod helpers;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            EntiTilesPlugin,
            EntiTilesHelpersPlugin::default(),
            PhysicsPlugins::default(),
            PhysicsDebugPlugin::default(),
        ))
        .insert_resource(LdtkPatterns::new(
            (0..=5)
                .into_iter()
                .map(|i| format!("World_Level_{}", i))
                .collect(),
            UVec2::splat(16),
        ))
        .insert_resource(LdtkLoadConfig {
            file_path: "assets/ldtk/wfc_source.ldtk".to_string(),
            asset_path_prefix: "ldtk/".to_string(),
            filter_mode: FilterMode::Nearest,
            ..Default::default()
        })
        .insert_resource(LdtkAdditionalLayers {
            physics_layer: Some(LdtkPhysicsLayer {
                identifier: "PhysicsCollider".to_string(),
                air: 0,
                parent: "Patterns".to_string(),
                tiles: Some(HashMap::from([
                    (
                        1,
                        PhysicsTile {
                            rigid_body: true,
                            friction: Some(0.5),
                        },
                    ),
                    (
                        2,
                        PhysicsTile {
                            rigid_body: true,
                            friction: Some(0.8),
                        },
                    ),
                ])),
            }),
            ..Default::default()
        })
        .insert_gizmo_config(PhysicsGizmos::all(), GizmoConfig::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (player_control, load_level))
        .register_type::<Player>()
        .run();
}

#[derive(Component, Reflect)]
struct Player {
    pub level: IVec2,
}

#[derive(Component, Reflect)]
struct LevelChange(UVec2);

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    let rules = WfcRules::from_file("examples/ldtk_wfc_config.ron", TilemapType::Square);
    commands.spawn((
        WfcRunner::new(
            TilemapType::Square,
            rules,
            TileArea::new(IVec2::ZERO, UVec2 { x: 4, y: 4 }),
            None,
        ),
        // you can also switch this to MultiMap mode
        // which will apply the result on a single tilemap
        WfcSource::LdtkMapPattern(LdtkWfcMode::SingleMap),
    ));

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::WHITE,
                custom_size: Some(Vec2 { x: 8., y: 8. }),
                ..Default::default()
            },
            transform: Transform::from_xyz(0., 0., 1.),
            ..Default::default()
        },
        Player { level: IVec2::ZERO },
    ));

    commands.spawn(LevelChange(UVec2::ZERO));
}

// The rest of the code is used to control the player and load the level.
// It will only function if you are using MultiMap mode.

fn player_control(
    mut commands: Commands,
    mut query: Query<(&mut Transform, &mut Player)>,
    input: Res<ButtonInput<KeyCode>>,
) {
    let Ok((mut transform, mut player)) = query.get_single_mut() else {
        return;
    };
    if input.pressed(KeyCode::ArrowLeft) {
        transform.translation.x -= 1.;
    }
    if input.pressed(KeyCode::ArrowRight) {
        transform.translation.x += 1.;
    }
    if input.pressed(KeyCode::ArrowUp) {
        transform.translation.y += 1.;
    }
    if input.pressed(KeyCode::ArrowDown) {
        transform.translation.y -= 1.;
    }

    // 8. * 16. = tile size * pattern size
    let new_idx = (transform.translation.xy() / Vec2::splat(8. * 16.)).as_ivec2();
    if new_idx != player.level {
        player.level = new_idx;
        if new_idx.x >= 0 && new_idx.y >= 0 {
            commands.spawn(LevelChange(new_idx.as_uvec2()));
        }
    }
}

fn load_level(
    mut commands: Commands,
    query: Query<(Entity, &LevelChange)>,
    mut level_manager: ResMut<LdtkLevelManager>,
    wfc_manager: Res<LdtkWfcManager>,
) {
    query.iter().for_each(|(e, l)| {
        if let Some(ident) = wfc_manager.get_ident(l.0) {
            level_manager.switch_to(
                &mut commands,
                ident,
                Some(wfc_manager.get_translation(l.0.as_ivec2(), Vec2::splat(8.))),
            );
        }
        commands.entity(e).despawn();
    });
}
