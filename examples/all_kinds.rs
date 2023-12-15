use bevy::{
    app::{App, Startup},
    asset::AssetServer,
    core_pipeline::core_2d::Camera2dBundle,
    ecs::system::{Commands, Res},
    math::{UVec2, Vec2},
    render::render_resource::FilterMode,
    DefaultPlugins,
};
use bevy_entitiles::{
    math::FillArea,
    render::texture::{TilemapTexture, TilemapTextureDescriptor},
    tilemap::{
        map::TilemapBuilder,
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
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    generate_tilemap(
        &mut commands,
        TileType::Square,
        "test_square.png",
        UVec2 { x: 32, y: 32 },
        UVec2 { x: 16, y: 16 },
        Vec2::ZERO,
        &asset_server,
    );

    generate_tilemap(
        &mut commands,
        TileType::Isometric,
        "test_isometric.png",
        UVec2 { x: 32, y: 32 },
        UVec2 { x: 32, y: 16 },
        Vec2 { x: 64., y: 150. },
        &asset_server,
    );

    generate_tilemap(
        &mut commands,
        TileType::Hexagonal(40),
        "test_hexagonal.png",
        UVec2 { x: 64, y: 64 },
        UVec2 { x: 28, y: 56 },
        Vec2 { x: 0., y: 300. },
        &asset_server,
    );
}

fn generate_tilemap(
    commands: &mut Commands,
    ty: TileType,
    texture_path: &str,
    texture_size: UVec2,
    tile_size: UVec2,
    offset: Vec2,
    asset_server: &AssetServer,
) {
    let (tilemap_entity, mut tilemap) = TilemapBuilder::new(
        ty,
        UVec2 { x: 8, y: 8 },
        tile_size.as_vec2(),
        "".to_string(),
    )
    .with_texture(TilemapTexture::new(
        asset_server.load(texture_path.to_string()),
        TilemapTextureDescriptor {
            size: texture_size,
            tile_size,
            filter_mode: FilterMode::Nearest,
        },
    ))
    .with_render_chunk_size(4)
    .with_translation(offset)
    .build(commands);
    tilemap.fill_rect(
        commands,
        FillArea::full(&tilemap),
        &TileBuilder::new().with_layer(0, 0),
    );
    commands.entity(tilemap_entity).insert(tilemap);
}
