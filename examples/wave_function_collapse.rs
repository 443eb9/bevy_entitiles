use bevy::prelude::{App, Camera2dBundle, Commands, Startup, UVec2, Vec2};
use bevy_entitiles::tilemap::{TileType, TilemapBuilder};

fn main() {
    App::new().add_systems(Startup, setup).run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    let (tilemap_entity, tilemap) = TilemapBuilder::new(
        TileType::Square,
        UVec2 { x: 20, y: 20 },
        Vec2 { x: 32., y: 32. },
    )
    .build(&mut commands);

    // tilemap.fill_rect(&mut commands, UVec2 { x: 0, y: 0 }, None, tile_builder)
}
