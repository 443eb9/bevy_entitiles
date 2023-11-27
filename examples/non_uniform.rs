use bevy::{
    app::Update,
    asset::Assets,
    prelude::{App, AssetServer, Camera2dBundle, Commands, Res, Startup, UVec2, Vec2},
    render::{
        mesh::{shape, Mesh},
        render_resource::FilterMode,
    },
    sprite::{ColorMaterial, ColorMesh2dBundle},
    DefaultPlugins,
};
use bevy_entitiles::{
    math::FillArea,
    render::texture::{TileUV, TileUVBuilder, TilemapTextureDescriptor},
    tilemap::{
        map::TilemapBuilder,
        tile::{TileBuilder, TileType},
    },
    EntiTilesPlugin,
};
use bevy_mod_outline::{AutoGenerateOutlineNormalsPlugin, OutlinePlugin};
use helpers::{
    drawing::{draw_axis, draw_chunk_aabb, draw_tilemap_aabb},
    EntiTilesDebugPlugin,
};

mod helpers;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            EntiTilesPlugin,
            EntiTilesDebugPlugin,
            OutlinePlugin,
            AutoGenerateOutlineNormalsPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (draw_axis, draw_tilemap_aabb, draw_chunk_aabb))
        .run();
}

fn setup(mut commands: Commands, assets_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let (tilemap_entity, mut tilemap) = TilemapBuilder::new(
        TileType::IsometricDiamond,
        UVec2 { x: 20, y: 10 },
        Vec2 { x: 32., y: 16. },
    )
    .with_texture(
        assets_server.load("test_nonuniform.png"),
        TilemapTextureDescriptor {
            tiles_uv: TileUVBuilder {
                image_size: UVec2 { x: 64, y: 80 },
                tiles: vec![
                    (UVec2 { x: 0, y: 0 }, UVec2 { x: 32, y: 32 }),
                    (UVec2 { x: 32, y: 8 }, UVec2 { x: 64, y: 32 }),
                    (UVec2 { x: 0, y: 32 }, UVec2 { x: 32, y: 59 }),
                    (UVec2 { x: 32, y: 32 }, UVec2 { x: 52, y: 69 }),
                    (UVec2 { x: 0, y: 64 }, UVec2 { x: 32, y: 80 }),
                ],
            }
            .build(),
            filter_mode: FilterMode::Nearest,
        },
    )
    // .with_tile_grid_size(Vec2 { x: 32., y: 16. })
    .with_anchor(Vec2 { x: 0.5, y: 0. })
    .with_render_chunk_size(4)
    .build(&mut commands);

    tilemap.fill_rect(
        &mut commands,
        FillArea::full(&tilemap),
        &TileBuilder::new(UVec2::ZERO, 4),
    );

    tilemap.fill_rect(
        &mut commands,
        FillArea::new(UVec2 { x: 2, y: 2 }, Some(UVec2 { x: 10, y: 7 }), &tilemap),
        &TileBuilder::new(UVec2::ZERO, 1),
    );

    tilemap.fill_rect(
        &mut commands,
        FillArea::new(UVec2 { x: 15, y: 5 }, Some(UVec2 { x: 4, y: 2 }), &tilemap),
        &TileBuilder::new(UVec2::ZERO, 2),
    );

    tilemap.set(&mut commands, &TileBuilder::new(UVec2 { x: 5, y: 5 }, 3));
    tilemap.set(&mut commands, &TileBuilder::new(UVec2 { x: 2, y: 6 }, 0));

    commands.entity(tilemap_entity).insert(tilemap);
}
