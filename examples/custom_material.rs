use bevy::{
    app::{App, Startup, Update},
    asset::{Asset, AssetServer, Assets},
    core_pipeline::core_2d::Camera2dBundle,
    ecs::system::{Commands, Res, ResMut},
    math::{IVec2, UVec2, Vec2},
    reflect::TypePath,
    render::render_resource::{AsBindGroup, FilterMode, ShaderRef},
    time::Time,
    DefaultPlugins,
};
use bevy_entitiles::{
    math::TileArea,
    render::material::{EntiTilesMaterialPlugin, TilemapMaterial},
    tilemap::{
        bundles::MaterialTilemapBundle,
        map::{
            TileRenderSize, TilemapRotation, TilemapSlotSize, TilemapStorage, TilemapTexture,
            TilemapTextureDescriptor, TilemapTextures,
        },
        tile::{TileBuilder, TileLayer},
    },
    EntiTilesPlugin, DEFAULT_CHUNK_SIZE,
};
use helpers::EntiTilesHelpersPlugin;

mod helpers;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            EntiTilesPlugin,
            EntiTilesHelpersPlugin::default(),
            // Don't forget to add the material plugin!
            EntiTilesMaterialPlugin::<MyMaterial>::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, update_time)
        .run();
}

#[derive(Asset, AsBindGroup, TypePath, Clone, Default)]
pub struct MyMaterial {
    #[uniform(0)]
    pub speed_and_time: Vec2,
}

impl TilemapMaterial for MyMaterial {
    fn fragment_shader() -> ShaderRef {
        "custom_material.wgsl".into()
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<MyMaterial>>,
    mut textures: ResMut<Assets<TilemapTextures>>,
) {
    commands.spawn(Camera2dBundle::default());

    let entity = commands.spawn_empty().id();
    let mut tilemap = MaterialTilemapBundle {
        tile_render_size: TileRenderSize(Vec2::splat(16.)),
        slot_size: TilemapSlotSize(Vec2::splat(16.)),
        textures: textures.add(TilemapTextures::single(
            TilemapTexture::new(
                asset_server.load("test_square.png"),
                TilemapTextureDescriptor::new(UVec2::splat(32), UVec2::splat(16)),
                TilemapRotation::None,
            ),
            FilterMode::Nearest,
        )),
        material: materials.add(MyMaterial {
            speed_and_time: Vec2::new(5., 0.),
        }),
        storage: TilemapStorage::new(DEFAULT_CHUNK_SIZE, entity),
        ..Default::default()
    };
    tilemap.storage.fill_rect(
        &mut commands,
        TileArea::new(IVec2::ZERO, UVec2::splat(5)),
        TileBuilder::new().with_layer(0, TileLayer::no_flip(0)),
    );
    commands.entity(entity).insert(tilemap);
}

fn update_time(mut materials: ResMut<Assets<MyMaterial>>, time: Res<Time>) {
    materials.iter_mut().for_each(|(_, material)| {
        material.speed_and_time[1] = time.elapsed_seconds() as f32;
    });
}
