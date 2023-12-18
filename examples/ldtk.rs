/*
 * NOTICE!
 * LDtk assets are not included!!
 */

use bevy::{
    app::{App, PluginGroup, Startup, Update},
    core_pipeline::core_2d::Camera2dBundle,
    ecs::{
        component::Component,
        event::EventReader,
        system::{Commands, Res, ResMut},
    },
    input::{keyboard::KeyCode, Input},
    render::{texture::ImagePlugin, view::Msaa},
    DefaultPlugins,
};
use bevy_entitiles::{
    math::FillArea,
    serializing::ldtk::{
        app_ext::AppExt, entities::LdtkEntity, enums::LdtkEnum, events::LdtkEvent,
        manager::LdtkLevelManager,
    },
    EntiTilesPlugin,
};
use bevy_entitiles_derive::{LdtkEntity, LdtkEnum};
use helpers::EntiTilesDebugPlugin;

mod helpers;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            EntiTilesPlugin,
            EntiTilesDebugPlugin,
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
            "assets/ldtk/grid_vania.ldtk".to_string(),
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
