use bevy::prelude::*;
use bevy_entitiles::prelude::*;
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

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardTilemapMaterial>>,
    mut textures: ResMut<Assets<TilemapTextures>>,
) {
    commands.spawn(Camera2dBundle::default());

    let entity = commands.spawn_empty().id();
    let mut tilemap = StandardTilemapBundle {
        tile_render_size: TileRenderSize(Vec2::new(16., 16.)),
        slot_size: TilemapSlotSize(Vec2::new(16., 16.)),
        ty: TilemapType::Square,
        storage: TilemapStorage::new(16, entity),
        material: materials.add(StandardTilemapMaterial::default()),
        textures: textures.add(TilemapTextures::single(
            TilemapTexture::new(
                asset_server.load("test_square.png"),
                TilemapTextureDescriptor::new(UVec2 { x: 32, y: 32 }, UVec2 { x: 16, y: 16 }),
            ),
            FilterMode::Nearest,
        )),
        ..Default::default()
    };

    let anim_a = tilemap.animations.register(RawTileAnimation {
        fps: 2,
        #[cfg(not(feature = "atlas"))]
        sequence: vec![0, 1, 2, 3],
        #[cfg(feature = "atlas")]
        sequence: vec![(0, 0), (0, 1), (0, 2), (0, 3)],
    });
    let anim_b = tilemap.animations.register(RawTileAnimation {
        fps: 3,
        #[cfg(not(feature = "atlas"))]
        sequence: vec![0, 1, 2],
        #[cfg(feature = "atlas")]
        sequence: vec![(0, 0), (0, 1), (0, 2)],
    });

    tilemap.storage.fill_rect(
        &mut commands,
        GridRect::new(IVec2::ZERO, UVec2 { x: 20, y: 20 }),
        TileBuilder::new().with_animation(anim_a),
    );

    tilemap.storage.fill_rect(
        &mut commands,
        GridRect::new(IVec2::ZERO, UVec2 { x: 10, y: 10 }),
        TileBuilder::new().with_animation(anim_b),
    );

    commands.entity(entity).insert(tilemap);
}
