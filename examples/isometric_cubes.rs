// To be honest, this crate is NOT suitable for 3d isometric tilemaps.
// Chunk sizes are must be n^2, so in this case, it must be 1^2, which is really
// inefficient. Maybe in the future, it will be allowed to set non-squre chunk sizes.

use bevy::prelude::*;
use bevy_entitiles::{prelude::*, render::chunk::RenderChunkSort};
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
        .add_systems(Update, rearrange)
        // Make sure you have specified the sorting method.
        .insert_resource(RenderChunkSort::XReverseThenYReverse)
        .run();
}

#[derive(Component)]
struct HintText;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardTilemapMaterial>>,
    mut textures: ResMut<Assets<TilemapTextures>>,
) {
    commands.spawn(Camera2dBundle::default());

    let entity = commands.spawn_empty().id();
    let mut tilemap = StandardTilemapBundle {
        tile_render_size: TileRenderSize(Vec2::splat(32.)),
        slot_size: TilemapSlotSize(Vec2::new(32., 16.)),
        ty: TilemapType::Isometric,

        // We can't take advantage of chunking for 3d isometric tilemaps.
        // Stacking tiles needs to do z test, and requires a depth texture for the pipeline.
        // But bevy doesn't support adding depth attachments to the pass in RenderCommand
        // storage: TilemapStorage::new(1, entity),

        // Here I want to demonstrate how to change the chunk_size after tilemap has spawned.
        // So I'll spawn a tilemap with a chunk_size of 32.
        storage: TilemapStorage::new(32, entity),

        material: materials.add(StandardTilemapMaterial::default()),
        textures: textures.add(TilemapTextures::single(
            TilemapTexture::new(
                asset_server.load("test_isometric_cubes.png"),
                TilemapTextureDescriptor::new(UVec2::new(64, 32), UVec2::splat(32)),
            ),
            FilterMode::Nearest,
        )),
        ..Default::default()
    };

    tilemap.storage.fill_rect_custom(
        &mut commands,
        GridRect::new(IVec2::ZERO, UVec2::splat(8)),
        |index| Some(TileBuilder::new().with_layer(0, TileLayer::no_flip(index.x % 2))),
        false,
    );

    commands.entity(entity).insert(tilemap);

    commands.spawn((
        HintText,
        TextBundle::from_section("Press Space to change chunk size.", TextStyle::default()),
    ));
}

fn rearrange(
    mut commands: Commands,
    mut tilemaps_query: Query<&mut TilemapStorage>,
    mut text: Query<&mut Text, With<HintText>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if input.just_pressed(KeyCode::Space) {
        for mut storage in &mut tilemaps_query {
            storage.rearrange(1, &mut commands);
        }

        text.single_mut().sections[0].value = "Chunk size changed!".to_owned();
    }
}
