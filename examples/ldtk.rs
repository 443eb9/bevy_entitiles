/*
 * NOTICE!
 * LDtk assets are not included!!
 */

use bevy::{
    app::{App, PluginGroup, Startup, Update},
    core_pipeline::core_2d::Camera2dBundle,
    ecs::{
        component::Component,
        system::{Commands, Query, Res}, bundle::Bundle,
    },
    input::{keyboard::KeyCode, Input},
    render::{render_resource::FilterMode, texture::ImagePlugin, view::Msaa},
    sprite::{TextureAtlas, TextureAtlasSprite},
    window::{Window, WindowPlugin},
    DefaultPlugins,
};
use bevy_entitiles::{
    serializing::ldtk::{app_ext::AppExt, entity::LdtkEntity, r#enum::LdtkEnum, LdtkLoader},
    tilemap::map::Tilemap,
    EntiTilesPlugin,
};
use bevy_entitiles_derive::{LdtkEntity, LdtkEnum};
use helpers::EntiTilesDebugPlugin;

mod helpers;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        present_mode: bevy::window::PresentMode::AutoNoVsync,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
            EntiTilesPlugin,
            EntiTilesDebugPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, load)
        // turn off msaa to avoid the white lines between tiles
        .insert_resource(Msaa::Off)
        .register_ldtk_entity::<Item>("Item")
        .register_ldtk_entity::<Player>("Player")
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn load(
    mut commands: Commands,
    input: Res<Input<KeyCode>>,
    mut tilemaps_query: Query<&mut Tilemap>,
) {
    if input.just_pressed(KeyCode::Space) {
        for mut map in tilemaps_query.iter_mut() {
            map.delete(&mut commands);
        }

        commands.spawn(LdtkLoader {
            path: "assets/ldtk/grid_vania.ldtk".to_string(),
            asset_path_prefix: "ldtk/".to_string(),
            at_depth: 0,
            filter_mode: FilterMode::Nearest,
            level: None,
            level_spacing: Some(30),
            ignore_unregistered_entities: true,
            use_tileset: Some(0),
            z_index: 0,
        });
    }
}

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
pub struct Player {
    // this is a wrapper which will be generated
    // when you derive LdtkEnum for your custom enums.
    // There are also another two wrappers:
    // ItemTypeOption and Item TypeOptionVec
    pub inventory: ItemTypeVec,
    pub hp: i32,
    // this will be deafult as it not exists in the ldtk file
    #[ldtk_default]
    pub mp: i32,
}

#[derive(Component, LdtkEntity)]
pub struct Item {
    pub ty: ItemType,
    pub price: i32,
    pub count: i32,
}
