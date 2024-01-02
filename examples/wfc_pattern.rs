use bevy::{
    app::{App, Startup},
    core_pipeline::core_2d::Camera2dBundle,
    ecs::{entity::Entity, system::Commands},
    math::{UVec2, Vec2, Vec4},
    DefaultPlugins,
};
use bevy_entitiles::{
    algorithm::wfc::{WfcRunner, WfcSource},
    math::TileArea,
    serializing::{
        save::{TilemapSaverBuilder, TilemapSaverMode},
        TilemapLayer,
    },
    tilemap::{
        layer::TileLayer,
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

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    // convert the image into 6 tilemaps as patterns
    let wfc_img = image::open("assets/test_wfc.png").unwrap().to_rgba8();

    const TILE_SIZE: u32 = 16;
    const PATTERN_SIZE: u32 = 16;
    const ROWS: u32 = 2;
    const COLS: u32 = 3;

    const PATTERNS_PATH: &str = "C:\\wfc_patterns";
    const PREFIX: &str = "wfc_pattern_";

    let mut tilemaps = [Entity::PLACEHOLDER; (COLS * ROWS) as usize];

    for row in 0..ROWS {
        for col in 0..COLS {
            let mut tilemap = TilemapBuilder::new(
                TileType::Square,
                UVec2 {
                    x: PATTERN_SIZE,
                    y: PATTERN_SIZE,
                },
                Vec2 { x: 8., y: 8. },
                format!("{}{}", PREFIX, col * ROWS + row),
            )
            .with_translation(Vec2 {
                x: (col * TILE_SIZE) as f32 * 8.,
                y: (row * TILE_SIZE) as f32 * -8. - 8. * PATTERN_SIZE as f32,
            })
            .with_render_chunk_size(16)
            .build(&mut commands);

            for y in 0..TILE_SIZE {
                for x in 0..TILE_SIZE {
                    let pixel = wfc_img.get_pixel(col * TILE_SIZE + x, row * TILE_SIZE + y);
                    tilemap.set(
                        &mut commands,
                        UVec2 {
                            x,
                            y: TILE_SIZE - y - 1,
                        },
                        TileBuilder::new()
                            .with_layer(0, TileLayer::new().with_texture_index(0))
                            .with_color(Vec4::new(
                                pixel[0] as f32 / 255.,
                                pixel[1] as f32 / 255.,
                                pixel[2] as f32 / 255.,
                                pixel[3] as f32 / 255.,
                            )),
                    );
                }
            }

            tilemaps[(col + row * COLS) as usize] = tilemap.id();
            commands.entity(tilemap.id()).insert(tilemap);
        }
    }

    tilemaps.iter().for_each(|map| {
        TilemapSaverBuilder::new(PATTERNS_PATH.to_string())
            .with_mode(TilemapSaverMode::MapPattern)
            .with_layer(TilemapLayer::Color)
            .remove_map_after_done()
            .build(&mut commands, *map);
    });

    // If you are running this example for the first time,
    // you need to comment the code below and run it once.
    // So the patterns are generated and saved to disk.

    let wfc_map = TilemapBuilder::new(
        TileType::Square,
        UVec2 { x: 80, y: 80 },
        Vec2 { x: 8., y: 8. },
        "wfc_map".to_string(),
    )
    .build(&mut commands);

    let rules = WfcRunner::read_rule_config(&wfc_map, "examples/wfc_config.ron".to_string());
    let mut area = TileArea::full(&wfc_map);
    area.set_extent(area.extent() / PATTERN_SIZE, &wfc_map);

    commands.entity(wfc_map.id()).insert((
        WfcSource::from_pattern_path(PATTERNS_PATH.to_string(), PREFIX.to_string(), &rules),
        WfcRunner::new(&wfc_map, rules, area, Some(0))
            .with_retrace_settings(Some(8), Some(1000000)),
    ));
}
