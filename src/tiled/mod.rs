use bevy::{
    app::{Plugin, Startup, Update},
    asset::{load_internal_asset, AssetServer, Assets, Handle},
    ecs::{
        entity::Entity,
        system::{Commands, Query, Res, ResMut},
    },
    render::{mesh::Mesh, render_resource::Shader, texture::Image},
    sprite::{Material2dPlugin, MaterialMesh2dBundle, Mesh2dHandle},
};

use self::{
    components::TiledLoader,
    resources::{TiledAssets, TiledLoadConfig, TiledTilemapManger},
    sprite::TiledSpriteMaterial,
    xml::layer::TiledLayer,
};

pub mod components;
pub mod resources;
pub mod sprite;
pub mod traits;
pub mod xml;

pub const TILED_SPRITE_SHADER: Handle<Shader> = Handle::weak_from_u128(13584136873461368486534);

pub struct EntiTilesTiledPlugin;

impl Plugin for EntiTilesTiledPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        load_internal_asset!(
            app,
            TILED_SPRITE_SHADER,
            "tiled_sprite.wgsl",
            Shader::from_wgsl
        );

        app.add_plugins(Material2dPlugin::<TiledSpriteMaterial>::default());

        app.add_systems(Startup, parse_tiled_xml);

        app.init_resource::<TiledLoadConfig>()
            .init_resource::<TiledAssets>()
            .init_resource::<TiledTilemapManger>();

        app.register_type::<TiledLoadConfig>()
            .register_type::<TiledAssets>()
            .register_type::<TiledTilemapManger>();

        app.add_systems(Update, load_tiled_xml);
    }
}

fn parse_tiled_xml(mut manager: ResMut<TiledTilemapManger>, config: Res<TiledLoadConfig>) {
    manager.reload_xml(&config);
}

fn load_tiled_xml(
    mut commands: Commands,
    loaders_query: Query<(Entity, &TiledLoader)>,
    mut manager: ResMut<TiledTilemapManger>,
    config: Res<TiledLoadConfig>,
    mut tiled_assets: ResMut<TiledAssets>,
    asset_server: Res<AssetServer>,
    mut material_assets: ResMut<Assets<TiledSpriteMaterial>>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
) {
    for (entity, loader) in &loaders_query {
        tiled_assets.initialize(
            &manager,
            &config,
            &asset_server,
            &mut material_assets,
            &mut mesh_assets,
        );

        load_tiled_tilemap(
            &mut commands,
            entity,
            &mut manager,
            &config,
            &tiled_assets,
            &asset_server,
            &loader,
        );

        commands.entity(entity).remove::<TiledLoader>();
    }
}

fn load_tiled_tilemap(
    commands: &mut Commands,
    tilemap_entity: Entity,
    manager: &mut TiledTilemapManger,
    config: &TiledLoadConfig,
    tiled_assets: &TiledAssets,
    asset_server: &AssetServer,
    loader: &TiledLoader,
) {
    let tiled_data = manager.get_cached_data().get(&loader.map).unwrap();

    tiled_data.xml.layers.iter().for_each(|layer| match layer {
        TiledLayer::Tiles(_) => {},
        TiledLayer::Objects(_) => {},
        TiledLayer::Image(layer) => {
            let (mesh, material) = (
                tiled_assets.clone_image_layer_mesh_handle(&tiled_data.name, layer.id),
                tiled_assets.clone_image_layer_material_handle(&tiled_data.name, layer.id),
            );

            commands.spawn(MaterialMesh2dBundle {
                mesh: Mesh2dHandle(mesh),
                material,
                ..Default::default()
            });
        }
        TiledLayer::Other => {}
    });
}
