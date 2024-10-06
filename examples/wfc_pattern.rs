use bevy::{
    app::{App, Startup},
    asset::Assets,
    color::LinearRgba,
    core_pipeline::core_2d::Camera2dBundle,
    ecs::{
        entity::Entity,
        system::{Commands, ResMut},
    },
    math::{IVec2, UVec2, Vec2},
    DefaultPlugins,
};
use bevy_entitiles::{
    algorithm::wfc::{WfcRules, WfcRunner, WfcSource},
    math::GridRect,
    render::material::StandardTilemapMaterial,
    serializing::map::{
        save::{TilemapSaver, TilemapSaverMode},
        TilemapLayer,
    },
    tilemap::{
        bundles::StandardPureColorTilemapBundle,
        map::{
            TileRenderSize, TilemapName, TilemapSlotSize, TilemapStorage, TilemapTransform,
            TilemapType,
        },
        tile::{TileBuilder, TileLayer},
    },
    EntiTilesPlugin,
};
use helpers::EntiTilesHelpersPlugin;

mod helpers;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            EntiTilesPlugin,
            EntiTilesHelpersPlugin::default(),
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, mut materials: ResMut<Assets<StandardTilemapMaterial>>) {
    commands.spawn(Camera2dBundle::default());

    // convert the image into 6 tilemaps as patterns
    let wfc_img = image::open("assets/test_wfc.png").unwrap().to_rgba8();

    const TILE_SIZE: u32 = 16;
    const PATTERN_SIZE: u32 = 16;
    const ROWS: u32 = 2;
    const COLS: u32 = 3;

    const PATTERNS_PATH: &str = "generated/wfc_pattern";
    const PREFIX: &str = "wfc_pattern_";

    let mut tilemaps = [Entity::PLACEHOLDER; (COLS * ROWS) as usize];

    for row in 0..ROWS {
        for col in 0..COLS {
            let entity = commands.spawn_empty().id();
            let mut tilemap = StandardPureColorTilemapBundle {
                name: TilemapName(format!("{}{}", PREFIX, col + row * COLS)),
                tile_render_size: TileRenderSize(Vec2::new(8., 8.)),
                slot_size: TilemapSlotSize(Vec2::new(8., 8.)),
                ty: TilemapType::Square,
                storage: TilemapStorage::new(16, entity),
                transform: TilemapTransform::from_translation(Vec2 {
                    x: (col * TILE_SIZE) as f32 * 8.,
                    y: (row * TILE_SIZE) as f32 * -8. - 8. * PATTERN_SIZE as f32,
                }),
                ..Default::default()
            };

            for y in 0..TILE_SIZE {
                for x in 0..TILE_SIZE {
                    let pixel = wfc_img.get_pixel(col * TILE_SIZE + x, row * TILE_SIZE + y);
                    tilemap.storage.set(
                        &mut commands,
                        IVec2 {
                            x: x as i32,
                            y: (PATTERN_SIZE - 1 - y) as i32,
                        },
                        TileBuilder::new()
                            .with_layer(0, TileLayer::no_flip(0))
                            .with_tint(LinearRgba::new(
                                pixel[0] as f32 / 255.,
                                pixel[1] as f32 / 255.,
                                pixel[2] as f32 / 255.,
                                pixel[3] as f32 / 255.,
                            )),
                    );
                }
            }

            tilemaps[(col + row * COLS) as usize] = entity;
            commands.entity(entity).insert(tilemap);
        }
    }

    tilemaps.into_iter().for_each(|map| {
        commands.entity(map).insert(TilemapSaver {
            path: PATTERNS_PATH.to_string(),
            mode: TilemapSaverMode::MapPattern,
            layers: TilemapLayer::COLOR,
            texture_path: None,
            remove_after_save: true,
        });
    });

    // If you are running this example for the first time,
    // you need to comment the code below and run it once.
    // So the patterns are generated and saved to disk.
    bevy::log::info!("=============================");
    bevy::log::info!("Program panicked? Look here!!");
    bevy::log::info!("=============================");

    let entity = commands.spawn_empty().id();

    let rules = WfcRules::from_file("examples/wfc_config.ron", TilemapType::Square);

    commands.entity(entity).insert((
        WfcSource::from_pattern_path(PATTERNS_PATH.to_string(), PREFIX.to_string(), &rules, None),
        WfcRunner::new(
            TilemapType::Square,
            rules,
            GridRect::new(IVec2::ZERO, UVec2 { x: 80, y: 80 } / PATTERN_SIZE),
            Some(0),
        ),
        StandardPureColorTilemapBundle {
            name: TilemapName("wfc_map".to_string()),
            tile_render_size: TileRenderSize(Vec2::new(8., 8.)),
            slot_size: TilemapSlotSize(Vec2::new(8., 8.)),
            ty: TilemapType::Square,
            storage: TilemapStorage::new(16, entity),
            material: materials.add(StandardTilemapMaterial::default()),
            ..Default::default()
        },
    ));
}
