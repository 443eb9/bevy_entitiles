use bevy::{
    asset::Assets,
    ecs::system::ResMut,
    prelude::{App, AssetServer, Camera2dBundle, Commands, Res, Startup, UVec2, Vec2},
    render::{
        color::Color,
        mesh::{shape::Circle, Mesh},
        render_resource::FilterMode,
    },
    sprite::{ColorMaterial, ColorMesh2dBundle},
    DefaultPlugins,
};
use bevy_entitiles::{
    math::FillArea,
    post_processing::{mist::FogData, PostProcessingView},
    render::texture::TilemapTextureDescriptor,
    tilemap::{
        map::TilemapBuilder,
        post_processing::height::HeightTilemap,
        tile::{TileBuilder, TileType},
    },
    EntiTilesPlugin,
};
use helpers::EntiTilesDebugPlugin;

mod helpers;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EntiTilesPlugin, EntiTilesDebugPlugin))
        .add_systems(Startup, setup)
        .insert_resource(FogData {
            min: 0.2,
            max: 5.,
            ..Default::default()
        })
        .run();
}

fn setup(
    mut commands: Commands,
    assets_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((Camera2dBundle::default(), PostProcessingView));

    commands.spawn(ColorMesh2dBundle {
        mesh: meshes.add(Circle::new(20.).into()).into(),
        material: materials.add(ColorMaterial::from(Color::hex("443EB9").unwrap())),
        ..Default::default()
    });

    let (tilemap_entity, mut tilemap) = TilemapBuilder::new(
        TileType::Square,
        UVec2 { x: 20, y: 10 },
        Vec2 { x: 32.0, y: 32.0 },
    )
    .with_texture(
        assets_server.load("test_square.png"),
        TilemapTextureDescriptor::from_full_grid(UVec2 { x: 2, y: 2 }, FilterMode::Nearest),
    )
    .build(&mut commands);

    tilemap.fill_rect(
        &mut commands,
        FillArea::full(&tilemap),
        &TileBuilder::new(UVec2::ZERO, 0),
    );

    tilemap.fill_rect(
        &mut commands,
        FillArea::new(UVec2 { x: 2, y: 2 }, Some(UVec2 { x: 10, y: 7 }), &tilemap),
        &TileBuilder::new(UVec2::ZERO, 1),
    );

    let height_tilemap = HeightTilemap::new(assets_server.load("height_texture.png"), &tilemap);

    commands
        .entity(tilemap_entity)
        .insert((tilemap, height_tilemap));
}
