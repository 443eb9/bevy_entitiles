use bevy::{
    app::{App, Startup, Update},
    core_pipeline::core_2d::Camera2dBundle,
    ecs::{
        component::Component,
        entity::Entity,
        query::With,
        system::{Commands, Query, Res},
    },
    input::{keyboard::KeyCode, Input},
    reflect::{DynamicStruct, Reflect},
    render::{render_resource::FilterMode, view::Msaa},
    DefaultPlugins,
};
use bevy_entitiles::{
    serializing::ldtk::{entities::LdtkEntity, json::level::EntityInstance, LdtkLoader},
    tilemap::map::Tilemap,
    EntiTilesPlugin,
};
use helpers::EntiTilesDebugPlugin;

mod helpers;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EntiTilesPlugin, EntiTilesDebugPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, load)
        // turn off msaa to avoid the white lines between tiles
        .insert_resource(Msaa::Off)
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
            asset_path_prefix: "ldtk".to_string(),
            at_depth: 0,
            filter_mode: FilterMode::Nearest,
            level_spacing: Some(30),
            tilemap_name: "ldtk".to_string(),
            scale: 1.,
            z_order: 0,
        });
    }
}

#[derive(Reflect)]
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

#[derive(Component, Reflect, Default)]
pub struct Player {
    pub inventory: Vec<ItemType>,
    pub hp: i32,
}

impl LdtkEntity for Player {
    fn spawn(data: &EntityInstance, dyn_struct: &mut DynamicStruct) -> Self {
        let mut s = Self::default();
        dyn_struct.apply(&mut s);
        s
    }
}

#[derive(Component, Reflect)]
pub struct Item {
    pub ty: ItemType,
    pub price: i32,
    pub count: i32,
}
