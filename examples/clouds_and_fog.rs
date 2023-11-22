use bevy::{
    prelude::{App, AssetServer, Camera2dBundle, Commands, Res, Startup, UVec2, Update, Vec2},
    render::render_resource::FilterMode,
    DefaultPlugins,
};
use bevy_entitiles::{
    debug::camera_movement::camera_control,
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

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EntiTilesPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, camera_control)
        .insert_resource(FogData { min: 0., max: 5. })
        .run();
}

fn setup(mut commands: Commands, assets_server: Res<AssetServer>) {
    commands.spawn((Camera2dBundle::default(), PostProcessingView));

    let (tilemap_entity, mut tilemap) = TilemapBuilder::new(
        TileType::Square,
        UVec2 { x: 20, y: 10 },
        Vec2 { x: 32.0, y: 32.0 },
    )
    .with_texture(
        assets_server.load("test/test_square.png"),
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

    let height_tilemap =
        HeightTilemap::new(assets_server.load("test/height_texture.png"), &tilemap);

    commands
        .entity(tilemap_entity)
        .insert((tilemap, height_tilemap));
}
