/*
 * NOTICE!
 * This feature was delayed! and this example cannot work!!!!
 */

use bevy::{
    app::Update,
    asset::Assets,
    math::Vec3,
    prelude::{App, AssetServer, Camera2dBundle, Commands, Res, Startup, UVec2, Vec2},
    render::{
        mesh::{shape, Mesh},
        render_resource::FilterMode,
        view::Msaa,
    },
    sprite::{ColorMaterial, ColorMesh2dBundle},
    DefaultPlugins,
};
use bevy_entitiles::{
    math::FillArea,
    post_processing::{
        mist::{FogData, FogLayer},
        PostProcessingSettings, PostProcessingView,
    },
    render::texture::{TileUV, TilemapTextureDescriptor},
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
        .insert_resource(PostProcessingSettings {
            filter_mode: FilterMode::Nearest,
            height_force_display: true,
        })
        .insert_resource(FogData {
            layers: [
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
            ],
            layer_count: 1,
            min: -0.,
            max: 1.,
            intensity: 1.,
            color: Vec3 {
                x: 0.,
                y: 0.,
                z: 0.,
            },
        })
        .insert_resource(Msaa::Off)
        .run();
}

fn setup(mut commands: Commands, assets_server: Res<AssetServer>) {
    commands.spawn((Camera2dBundle::default(), PostProcessingView));

    let (tilemap_entity, mut tilemap) = TilemapBuilder::new(
        TileType::Square,
        UVec2 { x: 20, y: 10 },
        Vec2 { x: 16., y: 16. },
    )
    .with_texture(
        assets_server.load("test_fog.png"),
        TilemapTextureDescriptor {
            size: UVec2 { x: 64, y: 64 },
            tiles_uv: vec![
                (UVec2 { x: 0, y: 0 }, UVec2 { x: 16, y: 16 }).into(),
                (UVec2 { x: 16, y: 0 }, UVec2 { x: 32, y: 32 }).into(),
                (UVec2 { x: 32, y: 0 }, UVec2 { x: 48, y: 48 }).into(),
                (UVec2 { x: 48, y: 0 }, UVec2 { x: 64, y: 64 }).into(),
            ],
            filter_mode: FilterMode::Nearest,
            is_uniform: false,
        },
    )
    .build(&mut commands);

    tilemap.fill_rect(
        &mut commands,
        FillArea::full(&tilemap),
        &TileBuilder::new(0),
    );

    tilemap.fill_rect(
        &mut commands,
        FillArea::new(UVec2 { x: 2, y: 2 }, Some(UVec2 { x: 10, y: 7 }), &tilemap),
        &TileBuilder::new(1),
    );

    tilemap.fill_rect(
        &mut commands,
        FillArea::new(UVec2 { x: 15, y: 5 }, Some(UVec2 { x: 4, y: 2 }), &tilemap),
        &TileBuilder::new(2),
    );

    tilemap.set(&mut commands, UVec2 { x: 5, y: 5 }, &TileBuilder::new(3));

    let height_tilemap = HeightTilemap::new(assets_server.load("test_fog_height.png"), &tilemap);

    commands
        .entity(tilemap_entity)
        .insert((tilemap, height_tilemap));
}
