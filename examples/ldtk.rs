/*
 * The icon set finalbossblues-icons_full_16 is not allowed to be redistributed.
 * So all those icons in the map will be white.
 */

use bevy::{
    app::{App, PluginGroup, Startup, Update},
    asset::AssetServer,
    core_pipeline::core_2d::Camera2dBundle,
    ecs::{
        component::Component,
        entity::Entity,
        event::EventReader,
        system::{Commands, EntityCommands, Res, ResMut},
    },
    input::{keyboard::KeyCode, Input},
    render::{texture::ImagePlugin, view::Msaa},
    utils::HashMap,
    DefaultPlugins,
};
use bevy_entitiles::{
    math::FillArea,
    serializing::ldtk::{
        app_ext::AppExt,
        entities::LdtkEntity,
        enums::LdtkEnum,
        events::LdtkEvent,
        json::{field::FieldInstance, level::EntityInstance},
        resources::{LdtkLevelManager, LdtkTextures},
    },
    EntiTilesPlugin,
};
use bevy_entitiles_derive::{LdtkEntity, LdtkEnum};
use bevy_xpbd_2d::{components::Collider, plugins::PhysicsDebugPlugin};
use helpers::EntiTilesDebugPlugin;

mod helpers;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            EntiTilesPlugin,
            EntiTilesDebugPlugin,
            PhysicsDebugPlugin::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (load, control, events))
        // turn off msaa to avoid the white lines between tiles
        .insert_resource(Msaa::Off)
        .register_ldtk_entity::<Item>("Item")
        .register_ldtk_entity::<Player>("Player")
        .run();
}

fn setup(mut commands: Commands, mut manager: ResMut<LdtkLevelManager>) {
    commands.spawn(Camera2dBundle::default());
    manager
        .initialize(
            // replace the filename with grid_vania.ldtk before running
            // this file uses finalbossblues-icons_full_16 and it only exists
            // in my local disk.
            "assets/ldtk/ignoregrid_vania.ldtk".to_string(),
            "ldtk/".to_string(),
        )
        .set_if_ignore_unregistered_entities(true);
}

fn control(input: Res<Input<KeyCode>>) {
    if input.just_pressed(KeyCode::Escape) {}
}

macro_rules! level_control {
    ($key:ident, $level:expr, $input:expr, $manager:expr, $commands:expr) => {
        if $input.pressed(KeyCode::ControlLeft) {
            if $input.just_pressed(KeyCode::$key) {
                $manager.unload(&mut $commands, $level);
            }
        } else if $input.just_pressed(KeyCode::$key) {
            $manager.try_load(&mut $commands, $level);
        }
    };
}

fn load(mut commands: Commands, input: Res<Input<KeyCode>>, mut manager: ResMut<LdtkLevelManager>) {
    level_control!(Key1, "Entrance", input, manager, commands);
    level_control!(Key2, "Cross_roads", input, manager, commands);
    level_control!(Key3, "Water_supply", input, manager, commands);
    level_control!(Key4, "Ossuary", input, manager, commands);
    level_control!(Key5, "Garden", input, manager, commands);
    level_control!(Key6, "Shop_entrance", input, manager, commands);
    level_control!(Key7, "phantom level", input, manager, commands);

    if input.just_pressed(KeyCode::Space) {
        manager.unload_all(&mut commands);
    }

    if input.just_pressed(KeyCode::Key0) {
        manager.try_load_many(
            &mut commands,
            &["Entrance", "Cross_roads", "Water_supply", "Ossuary"],
        );
    }

    if input.just_pressed(KeyCode::Key8) {
        manager.switch(&mut commands, "Entrance");
    }
}

fn events(mut ldtk_events: EventReader<LdtkEvent>) {
    for event in ldtk_events.read() {
        match event {
            LdtkEvent::LevelLoaded(level) => {
                println!("Level loaded: {}", level.identifier);
            }
            LdtkEvent::LevelUnloaded(level) => {
                println!("Level unloaded: {}", level.identifier);
            }
        }
    }
}

// this function will be called when the player entity is GOING TO BE spawned
// which means the entity still has no components
// you can consider this as a "extension"
// you don't need to impl the entire LdtkEntity trait but still able to do something
// that are not supported by generated code
fn player_spawn(
    // the level entity which will become the parent of this entity
    // if you don't use the #[global_entity] attribute
    _level_entity: Entity,
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
    // the textures from ldtk. They are already registered into assets.
    // you can use them to spawn new sprites.
    _ldtk_textures: &LdtkTextures,
) {
    // this is takes params that are exactly the same as the LdtkEntity trait
    // you can use this to add more fancy stuff to your entity
    // like adding a collider:
    commands.insert(Collider::cuboid(
        entity_instance.width as f32,
        entity_instance.height as f32,
    ));
}

// values here may show unreachable pattern warning
// It doesn't matter, and I don't know how to fix it
#[derive(LdtkEnum)]
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

#[derive(Component, LdtkEntity, Default)]
#[spawn_sprite]
// this means the entity will not disappear when the level is unloaded
#[global_entity]
#[callback(player_spawn)]
pub struct Player {
    // this is a wrapper which will be generated
    // when you derive LdtkEnum for your custom enums.
    // There are also another two wrappers:
    // ItemTypeOption and Item TypeOptionVec
    pub inventory: ItemTypeVec,
    #[ldtk_name = "HP"]
    pub hp: i32,
    // this will be deafult as it not exists in the ldtk file
    #[ldtk_default]
    pub mp: i32,
}

#[derive(Component, LdtkEntity)]
#[spawn_sprite]
pub struct Item {
    #[ldtk_name = "type"]
    pub ty: ItemType,
    pub price: i32,
    pub count: i32,
}
