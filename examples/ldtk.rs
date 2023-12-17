/*
 * NOTICE!
 * LDtk assets are not included!!
 */

use bevy::{
    app::{App, PluginGroup, Startup, Update},
    core_pipeline::core_2d::Camera2dBundle,
    ecs::{
        component::Component,
        system::{Commands, Res, ResMut},
    },
    input::{keyboard::KeyCode, Input},
    render::{texture::ImagePlugin, view::Msaa},
    window::{Window, WindowPlugin},
    DefaultPlugins,
};
use bevy_entitiles::{
    math::FillArea,
    serializing::ldtk::{
        app_ext::AppExt, entities::LdtkEntity, enums::LdtkEnum, manager::LdtkLevelManager,
        LdtkLevelIdent,
    },
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
        .add_systems(Update, (load, control))
        // turn off msaa to avoid the white lines between tiles
        .insert_resource(Msaa::Off)
        .register_ldtk_entity::<Item>("Item")
        .register_ldtk_entity::<Player>("Player")
        .run();
}

fn setup(
    mut commands: Commands,
    mut manager: ResMut<LdtkLevelManager>,
    asset_server: Res<bevy::asset::AssetServer>,
) {
    commands.spawn(Camera2dBundle::default());
    manager
        .initialize(
            "assets/ldtk/grid_vania.ldtk".to_string(),
            "ldtk/".to_string(),
        )
        .set_if_ignore_unregistered_entities(true);

    // let mut tilemap = bevy_entitiles::tilemap::map::TilemapBuilder::new(
    //     bevy_entitiles::tilemap::tile::TileType::Square,
    //     bevy::math::UVec2 { x: 8, y: 8 },
    //     bevy::math::Vec2 { x: 16., y: 16. },
    //     "".to_string(),
    // )
    // .with_texture(bevy_entitiles::render::texture::TilemapTexture::new(
    //     asset_server.load("test_square.png"),
    //     bevy_entitiles::render::texture::TilemapTextureDescriptor::new(
    //         bevy::math::UVec2 { x: 32, y: 32 },
    //         bevy::math::UVec2 { x: 16, y: 16 },
    //         bevy::render::render_resource::FilterMode::Nearest,
    //     ),
    // ))
    // .build(&mut commands);

    // tilemap.fill_rect(
    //     &mut commands,
    //     FillArea::full(&tilemap),
    //     &bevy_entitiles::tilemap::tile::TileBuilder::new().with_layer(0, 0),
    // );
    // commands.entity(tilemap.id()).insert(tilemap);
}

fn control(input: Res<Input<KeyCode>>) {
    if input.just_pressed(KeyCode::Escape) {}
}

macro_rules! level_control {
    ($key:ident, $level:expr, $input:expr, $manager:expr, $commands:expr) => {
        if $input.pressed(KeyCode::ControlLeft) {
            if $input.just_pressed(KeyCode::$key) {
                $manager.unload(&mut $commands, LdtkLevelIdent::Identifier($level));
            }
        } else if $input.just_pressed(KeyCode::$key) {
            $manager.load(&mut $commands, LdtkLevelIdent::Identifier($level));
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
#[spawn_sprite]
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
#[spawn_sprite]
pub struct Item {
    pub ty: ItemType,
    pub price: i32,
    pub count: i32,
}
