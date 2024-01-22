use bevy::{
    app::{App, Startup},
    core_pipeline::core_2d::Camera2dBundle,
    ecs::system::Commands,
    render::color::Color,
    DefaultPlugins,
};
use bevy_entitiles::{tiled::xml::TiledTilemap, EntiTilesPlugin};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EntiTilesPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    dbg!(quick_xml::de::from_str::<TiledTilemap>(
        // std::fs::read_to_string("assets/tiled/tilemaps/orthogonal.tmx")
        std::fs::read_to_string("assets/tiled/tilemaps/hexagonal.tmx")
            // std::fs::read_to_string("assets/tiled/tilemaps/isometric.tmx")
            .unwrap()
            .as_str(),
    )
    .unwrap());
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
