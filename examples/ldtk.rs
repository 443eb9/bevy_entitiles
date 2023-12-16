/*
 * NOTICE!
 * LDtk assets are not included!!
 * Because I don't know if I have the perimission to redistribute them.
 */

use bevy::{
    app::{App, PluginGroup, Startup, Update},
    asset::AssetServer,
    core_pipeline::core_2d::Camera2dBundle,
    ecs::{
        component::Component,
        system::{Commands, Query, Res},
    },
    input::{keyboard::KeyCode, Input},
    math::{Vec2, Vec3},
    render::{render_resource::FilterMode, texture::ImagePlugin, view::Msaa},
    sprite::{Sprite, SpriteBundle},
    transform::components::Transform,
    DefaultPlugins,
};
use bevy_entitiles::{
    serializing::ldtk::{
        app_ext::AppExt,
        entity::LdtkEntity,
        json::level::{EntityInstance, FieldValue},
        r#enum::LdtkEnum,
        LdtkLoader,
    },
    tilemap::map::Tilemap,
    EntiTilesPlugin,
};
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
            level: Some(0),
            level_spacing: Some(30),
            tilemap_name: "ldtk".to_string(),
            ignore_unregistered_entities: true,
            use_tileset: Some(0),
            z_index: 0,
        });
    }
}

pub enum ItemType {
    Meat,
    Gold,
    GoldNuggets,
    Gem,
    GreenGem,
    HealingPotion,
    Spell,
    Armor,
    Bow,
    Ammo,
    FireBlade,
    VorpalBlade,
}

impl LdtkEnum for ItemType {
    fn get_identifier(ident: &str) -> Self {
        match ident {
            "Meat" => ItemType::Meat,
            "Gold" => ItemType::Gold,
            "GoldNuggets" => ItemType::GoldNuggets,
            "Gem" => ItemType::Gem,
            "Green_gem" => ItemType::GreenGem,
            "Healing_potion" => ItemType::HealingPotion,
            "Spell" => ItemType::Spell,
            "Armor" => ItemType::Armor,
            "Bow" => ItemType::Bow,
            "Ammo" => ItemType::Ammo,
            "Fire_blade" => ItemType::FireBlade,
            "Vorpal_blade" => ItemType::VorpalBlade,
            _ => panic!("Unknown item type: {}", ident),
        }
    }
}

#[derive(Component)]
pub struct Player {
    pub inventory: Vec<ItemType>,
    pub hp: i32,
}

// All of these annoying derive work will be replace by a #[derive(LdtkEntity)] macro in the future!
impl LdtkEntity for Player {
    fn initialize(
        commands: &mut bevy::ecs::system::EntityCommands,
        entity_instance: &EntityInstance,
        asset_server: &AssetServer,
    ) -> Self {
        commands.insert(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(16., 16.)),
                ..Default::default()
            },
            texture: asset_server.load("player.png"),
            transform: Transform::from_translation(Vec3::new(
                entity_instance.world_x as f32,
                -entity_instance.world_y as f32,
                10.,
            )),
            ..Default::default()
        });
        Player {
            inventory: {
                match entity_instance.field_instances[0].value.as_ref().unwrap() {
                    FieldValue::LocalEnumArray(arr) => {
                        let mut inventory = Vec::new();
                        for item in arr.1.iter() {
                            inventory.push(ItemType::get_identifier(&item));
                        }
                        inventory
                    }
                    _ => panic!("Expected array value for inventory"),
                }
            },
            hp: match entity_instance.field_instances[1].value.as_ref().unwrap() {
                FieldValue::Integer(i) => *i,
                _ => panic!("Expected integer value for hp"),
            },
        }
    }
}

#[derive(Component)]
pub struct Item {
    pub ty: ItemType,
    pub price: i32,
    pub count: i32,
}

impl LdtkEntity for Item {
    fn initialize(
        commands: &mut bevy::ecs::system::EntityCommands,
        entity_instance: &EntityInstance,
        _asset_server: &AssetServer,
    ) -> Self {
        commands.insert(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(16., 16.)),
                ..Default::default()
            },
            transform: Transform::from_translation(Vec3::new(
                entity_instance.world_x as f32,
                -entity_instance.world_y as f32,
                10.,
            )),
            ..Default::default()
        });
        Item {
            ty: match entity_instance.field_instances[0].value.as_ref().unwrap() {
                FieldValue::LocalEnum(e) => ItemType::get_identifier(&e.1),
                _ => panic!("Expected local enum value for type"),
            },
            price: match entity_instance.field_instances[1].value.as_ref().unwrap() {
                FieldValue::Integer(i) => *i,
                _ => panic!("Expected integer value for price"),
            },
            count: match entity_instance.field_instances[2].value.as_ref().unwrap() {
                FieldValue::Integer(i) => *i,
                _ => panic!("Expected integer value for count"),
            },
        }
    }
}
