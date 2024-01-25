use bevy::{
    app::{App, Startup},
    core_pipeline::core_2d::Camera2dBundle,
    ecs::system::{Commands, ResMut},
    render::color::Color,
    DefaultPlugins,
};
use bevy_entitiles::{
    tiled::{
        resources::{TiledLoadConfig, TiledTilemapManger},
        xml::TiledTilemap,
    },
    EntiTilesPlugin,
};
use helpers::EntiTilesHelpersPlugin;

mod helpers;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            EntiTilesPlugin,
            EntiTilesHelpersPlugin::default(),
        ))
        .add_systems(Startup, setup)
        .insert_resource(TiledLoadConfig {
            map_path: vec![
                "assets/tiled/tilemaps/hexagonal.tmx".to_string(),
                "assets/tiled/tilemaps/infinite.tmx".to_string(),
                "assets/tiled/tilemaps/orthogonal.tmx".to_string(),
                "assets/tiled/tilemaps/isometric.tmx".to_string(),
            ],
        })
        .run();
}

fn setup(mut commands: Commands, mut manager: ResMut<TiledTilemapManger>) {
    commands.spawn(Camera2dBundle::default());

    manager.switch_to(&mut commands, "hexagonal".to_string(), None);
    manager.switch_to(&mut commands, "infinite".to_string(), None);
    manager.switch_to(&mut commands, "orthogonal".to_string(), None);
    manager.switch_to(&mut commands, "isometric".to_string(), None);
}

pub struct AnotherSquare {
    pub ty: EntityType,
    pub hp: f32,
    pub level: i32,
    pub rigid_body: bool,
    pub target: u32,
    pub texture: String,
    pub tint: Color,
}

pub struct Block {
    pub name: String,
    pub hp: f32,
}

pub enum EntityType {
    Player,
    Enemy,
}
