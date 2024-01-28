use bevy::{
    app::{App, PluginGroup, Startup},
    core_pipeline::core_2d::Camera2dBundle,
    ecs::{
        bundle::Bundle,
        component::Component,
        system::{Commands, ResMut},
    },
    render::{color::Color, texture::ImagePlugin},
    DefaultPlugins,
};
use bevy_entitiles::{
    tiled::{
        app_ext::TiledApp,
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
            DefaultPlugins.set(ImagePlugin::default_nearest()),
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
            ignore_unregisterd_objects: true,
        })
        .register_tiled_object::<BlockBundle>("Block")
        .register_tiled_object::<PlainBlockBundle>("PlainBlock")
        .run();
}

fn setup(mut commands: Commands, mut manager: ResMut<TiledTilemapManger>) {
    commands.spawn(Camera2dBundle::default());

    // manager.switch_to(&mut commands, "hexagonal".to_string(), None);
    // manager.switch_to(&mut commands, "infinite".to_string(), None);
    // manager.switch_to(&mut commands, "orthogonal".to_string(), None);
    manager.switch_to(&mut commands, "isometric".to_string(), None);
}

#[derive(Bundle)]
pub struct PlainBlockBundle {
    pub block: PlainBlock,
}

impl bevy_entitiles::tiled::traits::TiledObject for PlainBlockBundle {
    fn initialize(
        commands: &mut bevy::ecs::system::EntityCommands,
        object_instance: &bevy_entitiles::tiled::xml::layer::TiledObjectInstance,
        components: &bevy::utils::HashMap<
            String,
            bevy_entitiles::tiled::xml::property::ClassInstance,
        >,
        asset_server: &bevy::prelude::AssetServer,
        tiled_assets: &bevy_entitiles::tiled::resources::TiledAssets,
        tiled_map: String,
    ) {
        if object_instance.visible {
            let (mesh, z) = tiled_assets.clone_object_mesh_handle(&tiled_map, object_instance.id);
            commands.insert(bevy::sprite::MaterialMesh2dBundle {
                material: tiled_assets.clone_object_material_handle(&tiled_map, object_instance.id),
                mesh: bevy::sprite::Mesh2dHandle(mesh),
                transform: bevy::transform::components::Transform::from_xyz(
                    object_instance.x,
                    -object_instance.y,
                    z,
                ),
                ..Default::default()
            });
        }

        commands.insert(PlainBlock);
    }
}

#[derive(Component)]
pub struct PlainBlock;

#[derive(Bundle)]
pub struct BlockBundle {
    pub block: Block,
}

#[derive(Component)]
pub struct Block {
    pub collision: bool,
    pub hardness: f32,
    pub name: String,
    pub tint: Color,
    pub shape: ShapeType,
}

impl bevy_entitiles::tiled::traits::TiledObject for BlockBundle {
    fn initialize(
        commands: &mut bevy::ecs::system::EntityCommands,
        object_instance: &bevy_entitiles::tiled::xml::layer::TiledObjectInstance,
        components: &bevy::utils::HashMap<
            String,
            bevy_entitiles::tiled::xml::property::ClassInstance,
        >,
        asset_server: &bevy::prelude::AssetServer,
        tiled_assets: &bevy_entitiles::tiled::resources::TiledAssets,
        tiled_map: String,
    ) {
        if object_instance.visible {
            let (mesh, z) = tiled_assets.clone_object_mesh_handle(&tiled_map, object_instance.id);
            commands.insert(bevy::sprite::MaterialMesh2dBundle {
                material: tiled_assets.clone_object_material_handle(&tiled_map, object_instance.id),
                mesh: bevy::sprite::Mesh2dHandle(mesh),
                transform: bevy::transform::components::Transform::from_xyz(
                    object_instance.x,
                    -object_instance.y,
                    z,
                ),
                ..Default::default()
            });
        }

        commands.insert(Block {
            collision: components["Block"].properties["Collision"].clone().into(),
            hardness: components["Block"].properties["Hardness"].clone().into(),
            name: components["Block"].properties["Name"].clone().into(),
            tint: components["Block"].properties["Tint"].clone().into(),
            shape: components["Block"].properties["Shape"].clone().into(),
        });
    }
}

pub enum ShapeType {
    Square,
    Isometry,
    Hexagon,
    Polygon,
    Eclipse,
}

impl bevy_entitiles::tiled::traits::TiledEnum for ShapeType {
    fn get_identifier(ident: &str) -> Self {
        match ident {
            "Square" => ShapeType::Square,
            "Isometry" => ShapeType::Isometry,
            "Hexagon" => ShapeType::Hexagon,
            "Polygon" => ShapeType::Polygon,
            "Eclipse" => ShapeType::Eclipse,
            _ => panic!("Unknown enum variant: {}", ident),
        }
    }
}

impl Into<ShapeType> for bevy_entitiles::tiled::xml::property::PropertyInstance {
    fn into(self) -> ShapeType {
        match self.value {
            bevy_entitiles::tiled::xml::property::PropertyValue::Enum(_, x) => {
                <ShapeType as bevy_entitiles::tiled::traits::TiledEnum>::get_identifier(&x)
            }
            _ => panic!("Expected Enum value!"),
        }
    }
}
