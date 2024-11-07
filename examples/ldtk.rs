// The icon set finalbossblues-icons_full_16 is not allowed to be redistributed.
// So all those icons in the map will be white.
// If you are using the LDtk maps from the tutorials, you need to delete the internal
// icons tileset. Otherwise the program will panic due to the missing asset.

use avian2d::prelude::*;
use bevy::{
    ecs::system::EntityCommands, prelude::*, render::render_resource::FilterMode, utils::HashMap,
};
use bevy_entitiles::{
    ldtk::{
        json::{field::FieldInstance, level::EntityInstance},
        layer::physics::LdtkPhysicsLayer,
        resources::LdtkAdditionalLayers,
    },
    prelude::*,
};
use bevy_entitiles_derive::{LdtkEntity, LdtkEntityTag, LdtkEnum};
use helpers::EntiTilesHelpersPlugin;

mod helpers;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            EntiTilesPlugin,
            EntiTilesHelpersPlugin::default(),
            PhysicsPlugins::default(),
            PhysicsDebugPlugin::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                load,
                // hot_reload,
                player_control,
                physics_tile_events,
            ),
        )
        .register_type::<Teleport>()
        .register_type::<Player>()
        .register_type::<Item>()
        // turn off msaa to avoid the white lines between tiles
        .insert_resource(Msaa::Off)
        .insert_resource(Gravity(Vec2::new(0., -98.)))
        .insert_resource(LdtkLevelConfig {
            asset_path_prefix: "ldtk/".to_string(),
            filter_mode: FilterMode::Nearest,
            ignore_unregistered_entities: true,
            animation_mapper: HashMap::from([(
                470,
                RawTileAnimation {
                    #[cfg(feature = "atlas")]
                    sequence: vec![(0, 469), (0, 446), (0, 447)],
                    #[cfg(not(feature = "atlas"))]
                    sequence: vec![469, 446, 447],
                    fps: 3,
                },
            )]),
            ..Default::default()
        })
        .insert_resource(LdtkAdditionalLayers {
            physics_layer: Some(LdtkPhysicsLayer {
                identifier: "PhysicsColliders".to_string(),
                air: 0,
                parent: "Collisions".to_string(),
                tiles: Some(HashMap::from([
                    (
                        1,
                        PhysicsTile {
                            rigid_body: true,
                            friction: Some(0.9),
                        },
                    ),
                    (
                        2,
                        PhysicsTile {
                            rigid_body: true,
                            friction: Some(0.1),
                        },
                    ),
                ])),
            }),
            ..Default::default()
        })
        .insert_gizmo_config(PhysicsGizmos::all(), GizmoConfig::default())
        .register_ldtk_entity::<Item>("Item")
        .register_ldtk_entity::<Player>("Player")
        .register_ldtk_entity::<Teleport>("Teleport")
        .register_ldtk_entity::<Ladder>("Ladder")
        .register_ldtk_entity::<SecretArea>("SecretArea")
        .register_ldtk_entity_tag::<Actor>("actor")
        .register_ldtk_entity_tag::<Loot>("loot")
        .register_ldtk_entity_tag::<Region>("region")
        .run();
}

#[derive(Resource, Deref)]
struct LdtkFile(Handle<LdtkJson>);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    commands.insert_resource(LdtkFile(asset_server.load("ldtk/ignore grid_vania.ldtk")));
    commands.spawn(TextBundle::from_section(
        "Press digit from 1-7 to load levels.\n\
        Press space to unload all levels.",
        Default::default(),
    ));
}

macro_rules! level_control {
    ($key:ident, $level:expr, $input:expr, $file:expr, $event:expr) => {
        if $input.pressed(KeyCode::ControlLeft) {
            if $input.just_pressed(KeyCode::$key) {
                $event.send(LdtkLevelEvent::Unload(LdtkLevelUnloader {
                    json: $file.id(),
                    level: LdtkLevel::Identifier($level.into()),
                }));
            }
        } else if $input.just_pressed(KeyCode::$key) {
            $event.send(LdtkLevelEvent::Load(LdtkLevelLoader {
                json: $file.id(),
                level: LdtkLevel::Identifier($level.into()),
                mode: LdtkLevelLoaderMode::Tilemap,
                trans_ovrd: None,
            }));
        }
    };
}

fn load(
    input: Res<ButtonInput<KeyCode>>,
    mut event: EventWriter<LdtkLevelEvent>,
    file: Res<LdtkFile>,
    loaded_level: Res<LdtkLoadedLevels>,
) {
    level_control!(Digit1, "Entrance", input, file, event);
    level_control!(Digit2, "Cross_roads", input, file, event);
    level_control!(Digit3, "Water_supply", input, file, event);
    level_control!(Digit4, "Ossuary", input, file, event);
    level_control!(Digit5, "Garden", input, file, event);
    level_control!(Digit6, "Shop_entrance", input, file, event);
    level_control!(Digit7, "phantom level", input, file, event);

    if input.just_pressed(KeyCode::Space) {
        loaded_level.unload_all_at(file.id(), &mut event);
    }
}

fn player_control(
    mut query: Query<&mut LinearVelocity, With<Player>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    let Ok(mut player) = query.get_single_mut() else {
        return;
    };
    // wasd is taken up by the camera controller.
    if input.pressed(KeyCode::ArrowLeft) {
        player.x = -30.;
    }
    if input.pressed(KeyCode::ArrowRight) {
        player.x = 30.;
    }
    // I know this is not scientifically correct
    // because the player will be able to jump infinitely
    // but I'm lazy to do the detection :p
    if input.pressed(KeyCode::ArrowUp) {
        player.y = 100.;
    }
}

// this function will be called when the player entity is GOING TO BE spawned
// which means the entity still has no components
// you can consider this as a "extension"
// you don't need to impl the entire LdtkEntity trait but still able to do something
// that are not supported by generated code
fn player_spawn(
    // the entity commands for the entity
    commands: &mut EntityCommands,
    // all the data from ldtk
    entity_instance: &EntityInstance,
    // the fields of this entity you can access them using their identifiers(ldtk side)
    // generally you don't need to use this, and this fields will be applyed to the entity later
    // with generated code
    _fields: &HashMap<String, FieldInstance>,
    // the asset server
    _asset_server: &AssetServer,
    // the ldtk assets, like sprites and meshes
    _ldtk_assets: &LdtkAssets,
) {
    // this is takes params that are exactly the same as the LdtkEntity trait
    // you can use this to add more fancy stuff to your entity
    // like adding a collider:
    let size = Vec2::new(entity_instance.width as f32, entity_instance.height as f32);
    commands.insert((
        Collider::convex_hull(vec![
            Vec2::new(-0.5, 0.) * size,
            Vec2::new(0.5, 0.) * size,
            Vec2::new(0.5, 1.) * size,
            Vec2::new(-0.5, 1.) * size,
        ])
        .unwrap(),
        RigidBody::Dynamic,
        Friction {
            dynamic_coefficient: 0.5,
            static_coefficient: 0.5,
            ..Default::default()
        },
        Mass(100.),
    ));
}

#[derive(LdtkEnum, Reflect, Clone, Copy, Debug)]
#[wrapper_derive(Reflect, Default)]
pub enum ItemType {
    Meat,
    Gold,
    GoldNuggets,
    Gem,
    #[ldtk_name = "Green_gem"]
    GreenGem,
    #[ldtk_name = "Healing_potion"]
    HealingPotion,
    Spell,
    Armor,
    Bow,
    Ammo,
    #[ldtk_name = "Fire_blade"]
    FireBlade,
    #[ldtk_name = "Vorpal_blade"]
    VorpalBlade,
}

#[derive(Component, LdtkEntity, Default, Reflect)]
// this means the entity will be spawned with a sprite
#[spawn_sprite]
// this means the entity will not disappear when the level is unloaded
#[global_entity]
#[callback(player_spawn)]
pub struct Player {
    // this is a wrapper which will be generated
    // when you derive LdtkEnum for your custom enums.
    // There are also another two wrappers:
    // ItemTypeOption and Item TypeOptionVec

    // As impl a foreign trait for a foreign type is not allowed in rust,
    // we have to define these two wrappers.

    // You can impl the LdtkEntity trait yourself so these wrappers
    // can be avoided.
    pub inventory: ItemTypeVec,
    #[ldtk_name = "HP"]
    pub hp: i32,
    // this will be deafult as it not exists in the ldtk file
    #[ldtk_default]
    pub mp: i32,
}

#[derive(Component, LdtkEntity, Reflect)]
#[spawn_sprite]
pub struct Ladder;

#[derive(Component, LdtkEntity, Reflect)]
#[spawn_sprite]
pub struct SecretArea;

#[derive(Component, LdtkEntity, Reflect)]
#[spawn_sprite]
pub struct Item {
    #[ldtk_name = "type"]
    pub ty: ItemType,
    pub price: i32,
    pub count: i32,
}

#[derive(Component, LdtkEntity, Reflect)]
#[spawn_sprite]
pub struct Teleport {
    pub destination: bevy_entitiles::ldtk::json::EntityRef,
}

// Marker components for generated physics tiles.

#[derive(Component)]
pub struct Wall;

#[derive(Component)]
pub struct PoolBottom;

fn physics_tile_events(mut commands: Commands, mut event: EventReader<PhysicsTileSpawn>) {
    event.read().for_each(|ev| {
        match ev.int_repr {
            Some(1) => commands.entity(ev.tile).insert(Wall),
            Some(2) => commands.entity(ev.tile).insert(PoolBottom),
            _ => unreachable!(),
        };
    });
}

// Entity tags are the tags in PROJECT ENTITIES -> ENTITY SETTINGS
// -> Tags in LDtk
#[derive(Component, LdtkEntityTag)]
pub struct Actor;

#[derive(Component, LdtkEntityTag)]
pub struct Loot;

#[derive(Component, LdtkEntityTag)]
pub struct Region;
