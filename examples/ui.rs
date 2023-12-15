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
    render::{
        buffer::TileAnimation,
        texture::{TilemapTexture, TilemapTextureDescriptor},
    },
    tilemap::tile::TileFlip,
    ui::{UiTileBuilder, UiTileMaterial, UiTileMaterialRegistry},
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
    mut mat_reg: ResMut<UiTileMaterialRegistry>,
) {
    commands.spawn(Camera2dBundle::default());

    let texture = TilemapTexture::new(
        asset_server.load("test_square.png"),
        TilemapTextureDescriptor::new(
            UVec2 { x: 32, y: 32 },
            UVec2 { x: 16, y: 16 },
            FilterMode::Nearest,
        ),
    );

    // call this if you want to register all static atlas tiles.
    // let builders = UiTileBuilder::new().fill_grid_with_atlas(texture.desc());
    // and use register_many

    mat_reg.register(
        &mut materials,
        &texture,
        &UiTileBuilder::new().with_animation(TileAnimation::new(vec![0, 1, 2, 3], 5.)),
    );

    mat_reg.register(
        &mut materials,
        &texture,
        &UiTileBuilder::new()
            .with_texture_index(0)
            .with_color(Color::CYAN.into())
            .with_flip(TileFlip::Horizontal),
    );

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
                material: mat_reg.get_handle(texture.handle(), 0).unwrap(),
                ..Default::default()
            });
            root.spawn(MaterialNodeBundle {
                style: Style {
                    width: Val::Px(200.),
                    height: Val::Px(200.),
                    ..Default::default()
                },
                material: mat_reg.get_handle(texture.handle(), 1).unwrap(),
                ..Default::default()
            });
        });
}
