use bevy::{
    app::{App, PluginGroup, Startup, Update},
    asset::AssetServer,
    core_pipeline::core_2d::Camera2dBundle,
    ecs::{
        component::Component,
        entity::Entity,
        event::EventReader,
        query::{With, Without},
        system::{Commands, Query, Res},
    },
    input::{keyboard::KeyCode, Input},
    math::{IVec2, UVec2, Vec2},
    render::render_resource::FilterMode,
    text::{Text, TextSection, TextStyle},
    ui::{node_bundles::TextBundle, JustifySelf, PositionType, Style, Val},
    window::{PresentMode, ReceivedCharacter, Window, WindowPlugin},
    DefaultPlugins,
};
use bevy_entitiles::{
    debug::EntiTilesDebugPlugin,
    math::TileArea,
    serializing::chunk::save::{TilemapChunkSaver, TilemapPathChunkSaver},
    tilemap::{
        algorithm::path::{PathTile, PathTilemap},
        bundles::TilemapBundle,
        map::{
            TileRenderSize, TilemapName, TilemapRotation, TilemapSlotSize, TilemapStorage,
            TilemapTexture, TilemapTextureDescriptor, TilemapType,
        },
        tile::{TileBuilder, TileLayer},
    },
    EntiTilesPlugin,
};
use helpers::EntiTilesHelpersPlugin;

mod helpers;

const CHUNK_SIZE: u32 = 32;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    present_mode: PresentMode::Immediate,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            EntiTilesPlugin,
            EntiTilesDebugPlugin,
            EntiTilesHelpersPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (manual_unload, keyboard_input))
        .run()
}

#[derive(Component)]
struct ChunkIndexInput;

#[derive(Component)]
struct InfoDisplay;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let entity = commands.spawn_empty().id();
    let mut tilemap = TilemapBundle {
        name: TilemapName("test_map".to_string()),
        tile_render_size: TileRenderSize(Vec2::new(16., 16.)),
        slot_size: TilemapSlotSize(Vec2::new(16., 16.)),
        ty: TilemapType::Square,
        storage: TilemapStorage::new(CHUNK_SIZE, entity),
        texture: TilemapTexture::new(
            asset_server.load("test_square.png"),
            TilemapTextureDescriptor::new(
                UVec2 { x: 32, y: 32 },
                UVec2 { x: 16, y: 16 },
                FilterMode::Nearest,
            ),
            TilemapRotation::None,
        ),
        ..Default::default()
    };

    tilemap.storage.fill_rect(
        &mut commands,
        TileArea::new(IVec2 { x: -250, y: -250 }, UVec2 { x: 500, y: 500 }),
        TileBuilder::new().with_layer(0, TileLayer::new().with_texture_index(0)),
    );

    let mut path_tilemap = PathTilemap::new_with_chunk_size(CHUNK_SIZE);
    path_tilemap.fill_path_rect_custom(
        TileArea::new(IVec2 { x: -250, y: -250 }, UVec2 { x: 500, y: 500 }),
        |_| {
            Some(PathTile {
                cost: rand::random::<u32>() % 10,
            })
        },
    );

    commands.entity(entity).insert((tilemap, path_tilemap));

    commands.spawn((
        TextBundle::from_sections([
            TextSection {
                value:
                    "Enter the index of the chunk (eg: 0_0, 0_0~1_2)\nAnd press enter to unload\n"
                        .to_string(),
                style: TextStyle {
                    font_size: 30.,
                    ..Default::default()
                },
            },
            TextSection {
                value: "".to_string(),
                style: TextStyle {
                    font_size: 30.,
                    ..Default::default()
                },
            },
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(30.),
            justify_self: JustifySelf::Center,
            ..Default::default()
        }),
        InfoDisplay,
    ));

    commands.spawn((
        TextBundle::from_section(
            "",
            TextStyle {
                font_size: 30.,
                ..Default::default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(30.),
            justify_self: JustifySelf::Center,
            ..Default::default()
        }),
        ChunkIndexInput,
    ));
}

fn keyboard_input(
    mut text: Query<&mut Text, With<ChunkIndexInput>>,
    mut ev: EventReader<ReceivedCharacter>,
) {
    ev.read().for_each(|input| {
        if input.char == '\r' {
            return;
        }
        text.single_mut().sections[0].value.push(input.char);
    });
}

fn manual_unload(
    mut commands: Commands,
    tilemaps_query: Query<Entity, With<TilemapStorage>>,
    mut text: Query<&mut Text, (With<ChunkIndexInput>, Without<InfoDisplay>)>,
    mut info: Query<&mut Text, (With<InfoDisplay>, Without<ChunkIndexInput>)>,
    keyboard: Res<Input<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Return) {
        let mut info = info.single_mut();
        let mut text = text.single_mut();
        let value = text.sections[0].value.clone();

        let mut col_saver = TilemapChunkSaver::new("C:\\maps".to_string()).remove_after_save();
        let mut path_saver = TilemapPathChunkSaver::new("C:\\maps".to_string()).remove_after_save();

        // some low quality trash
        // =============================================================
        {
            let mut iter_mul = value.split('~');
            let count = iter_mul.clone().count();

            if count == 2 {
                let mut bounds = [IVec2::ZERO; 2];
                for i in 0..=1 {
                    let cur = if let Some(input) = iter_mul.next() {
                        input
                    } else {
                        fail(&mut info, &value, &mut text);
                        return;
                    };

                    if let Some(idx) = parse_index(cur) {
                        bounds[i] = idx;
                    } else {
                        fail(&mut info, &value, &mut text);
                        return;
                    }
                }
                // BUT THESE 2 LINES ARE IMPORTANT!!!!!!
                col_saver = col_saver.with_range(bounds[0], bounds[1]);
                path_saver = path_saver.with_range(bounds[0], bounds[1]);
            } else if count == 1 {
                let Some(idx) = parse_index(&value) else {
                    fail(&mut info, &value, &mut text);
                    return;
                };
                // ALSO THESE 2!!!
                col_saver = col_saver.with_single(idx);
                path_saver = path_saver.with_single(idx);
            } else {
                fail(&mut info, &value, &mut text);
                return;
            };
        }
        // just to parse your input don't mind this trash
        // I put them into a code block so you can fold and ignore them :)
        // =============================================================

        text.sections[0].value = "".to_string();
        commands
            .entity(tilemaps_query.single())
            .insert((col_saver, path_saver));
    }
}

fn parse_index(value: &str) -> Option<IVec2> {
    let mut iter_sig = value.split('_');
    let mut xy = IVec2::ZERO;

    for i in 0..=1 {
        if let Some(e) = iter_sig.next() {
            if let Ok(e) = e.parse() {
                xy[i] = e;
            } else {
                return None;
            }
        } else {
            return None;
        }
    }

    Some(xy)
}

fn fail(info: &mut Text, value: &str, text: &mut Text) {
    info.sections[1].value = format!("Invalid input:\n{}", value);
    text.sections[0].value = "".to_string();
}
