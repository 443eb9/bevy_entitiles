use bevy::{
    app::{App, PluginGroup, Startup},
    asset::{AssetServer, Assets},
    core_pipeline::core_2d::Camera2dBundle,
    ecs::system::{Commands, Res, ResMut},
    hierarchy::BuildChildren,
    math::UVec2,
    render::{color::Color, render_resource::FilterMode, texture::ImagePlugin},
    ui::{
        node_bundles::{MaterialNodeBundle, NodeBundle},
        Style, Val,
    },
    DefaultPlugins,
};
use bevy_entitiles::{
    render::texture::{TilemapTexture, TilemapTextureDescriptor},
    ui::UiTileMaterial,
    EntiTilesPlugin,
};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            EntiTilesPlugin,
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<UiTileMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());

    let textures = TilemapTexture::new(
        asset_server.load("test_square.png"),
        TilemapTextureDescriptor {
            size: UVec2 { x: 32, y: 32 },
            tile_size: UVec2 { x: 16, y: 16 },
            filter_mode: FilterMode::Nearest,
        },
    )
    .as_ui_texture()
    .register_materials(None, &mut materials);

    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                ..Default::default()
            },
            ..Default::default()
        })
        .with_children(|root| {
            root.spawn(MaterialNodeBundle {
                style: Style {
                    width: Val::Px(300.),
                    height: Val::Px(300.),
                    ..Default::default()
                },
                material: textures.clone(0).unwrap(),
                ..Default::default()
            });
            root.spawn(NodeBundle {
                style: Style {
                    width: Val::Px(200.),
                    height: Val::Px(200.),
                    ..Default::default()
                },
                background_color: Color::WHITE.into(),
                ..Default::default()
            });
        });
}
