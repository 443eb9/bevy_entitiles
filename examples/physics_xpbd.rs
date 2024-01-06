use bevy::{
    app::FixedUpdate,
    asset::Assets,
    ecs::{
        component::Component,
        event::EventReader,
        query::With,
        system::{Query, ResMut},
    },
    input::{keyboard::KeyCode, Input},
    math::{IVec2, Vec3},
    prelude::{App, AssetServer, Camera2dBundle, Commands, Res, Startup, UVec2, Update, Vec2},
    render::{
        color::Color,
        mesh::{shape::Circle, Mesh},
        render_resource::FilterMode,
    },
    sprite::{ColorMaterial, ColorMesh2dBundle},
    transform::components::Transform,
    DefaultPlugins,
};
use bevy_entitiles::{
    debug::EntiTilesDebugPlugin,
    math::TileArea,
    tilemap::{
        bundles::TilemapBundle,
        map::{
            TileRenderSize, TilemapName, TilemapRotation, TilemapSlotSize, TilemapStorage,
            TilemapTexture, TilemapTextureDescriptor, TilemapTransform, TilemapType,
        },
        physics::{xpbd, TileCollision},
        tile::{TileBuilder, TileLayer},
    },
    EntiTilesPlugin,
};
use bevy_xpbd_2d::{
    components::{Collider, LinearVelocity, RigidBody},
    plugins::{PhysicsDebugPlugin, PhysicsPlugins},
};
use helpers::EntiTilesHelpersPlugin;

mod helpers;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            EntiTilesPlugin,
            EntiTilesDebugPlugin,
            EntiTilesHelpersPlugin,
            PhysicsPlugins::default(),
            PhysicsDebugPlugin::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, collision_events)
        .add_systems(FixedUpdate, character_move)
        .run();
}

fn setup(
    mut commands: Commands,
    assets_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());

    let entity = commands.spawn_empty().id();
    let mut tilemap = TilemapBundle {
        name: TilemapName("test_map".to_string()),
        tile_render_size: TileRenderSize(Vec2::new(32., 16.)),
        slot_size: TilemapSlotSize(Vec2::new(32., 16.)),
        ty: TilemapType::Isometric,
        storage: TilemapStorage::new(16, entity),
        tilemap_transform: TilemapTransform::from_z_index(-1),
        texture: TilemapTexture::new(
            assets_server.load("test_isometric.png"),
            TilemapTextureDescriptor::new(
                UVec2 { x: 32, y: 32 },
                UVec2 { x: 32, y: 16 },
                FilterMode::Nearest,
            ),
            TilemapRotation::None,
        ),
        ..Default::default()
    };

    tilemap.storage.fill_rect(
        &mut commands,
        TileArea::new(IVec2::ZERO, UVec2 { x: 20, y: 10 }),
        TileBuilder::new().with_layer(0, TileLayer::new().with_texture_index(0)),
    );

    xpbd::set_physics_tile(
        &mut commands,
        IVec2 { x: 19, y: 9 },
        None,
        true,
        &tilemap.storage,
        &tilemap.ty,
        &tilemap.tilemap_transform,
        &tilemap.tile_pivot,
        &tilemap.slot_size,
    );

    xpbd::fill_physics_tile(
        &mut commands,
        TileArea::new(IVec2::ZERO, UVec2 { x: 5, y: 5 }),
        Some(0.8),
        false,
        &tilemap.storage,
        &tilemap.ty,
        &tilemap.tilemap_transform,
        &tilemap.tile_pivot,
        &tilemap.slot_size,
    );

    commands.entity(entity).insert(tilemap);

    let entity = commands.spawn_empty().id();
    let mut tilemap = TilemapBundle {
        name: TilemapName("test_map".to_string()),
        tile_render_size: TileRenderSize(Vec2::new(16., 16.)),
        slot_size: TilemapSlotSize(Vec2::new(16., 16.)),
        ty: TilemapType::Square,
        storage: TilemapStorage::new(16, entity),
        tilemap_transform: TilemapTransform::from_translation(Vec2::new(500., -100.)),
        texture: TilemapTexture::new(
            assets_server.load("test_square.png"),
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
        TileArea::new(IVec2::ZERO, UVec2 { x: 20, y: 10 }),
        TileBuilder::new().with_layer(0, TileLayer::new().with_texture_index(0)),
    );

    xpbd::set_physics_tile(
        &mut commands,
        IVec2 { x: 19, y: 9 },
        None,
        true,
        &tilemap.storage,
        &tilemap.ty,
        &tilemap.tilemap_transform,
        &tilemap.tile_pivot,
        &tilemap.slot_size,
    );

    xpbd::fill_physics_tile(
        &mut commands,
        TileArea::new(IVec2::ZERO, UVec2 { x: 5, y: 5 }),
        Some(0.8),
        false,
        &tilemap.storage,
        &tilemap.ty,
        &tilemap.tilemap_transform,
        &tilemap.tile_pivot,
        &tilemap.slot_size,
    );

    commands.entity(entity).insert(tilemap);

    // spawn a character
    commands.spawn((
        ColorMesh2dBundle {
            mesh: meshes.add(Circle::new(15.).into()).into(),
            material: materials.add(ColorMaterial::from(Color::WHITE)),
            transform: Transform::from_translation(Vec3::new(0., -50., 0.)),
            ..Default::default()
        },
        Collider::ball(15.),
        RigidBody::Dynamic,
        LinearVelocity::ZERO,
        Character,
    ));
}

fn collision_events(mut collision: EventReader<TileCollision>) {
    for c in collision.read() {
        println!("Collision: {:?}", c);
    }
}

#[derive(Component)]
pub struct Character;

pub fn character_move(
    input: Res<Input<KeyCode>>,
    mut query: Query<&mut LinearVelocity, With<Character>>,
) {
    let mut dir = Vec2::ZERO;
    if input.pressed(KeyCode::Up) {
        dir += Vec2::Y;
    }
    if input.pressed(KeyCode::Down) {
        dir -= Vec2::Y;
    }
    if input.pressed(KeyCode::Left) {
        dir -= Vec2::X;
    }
    if input.pressed(KeyCode::Right) {
        dir += Vec2::X;
    }
    for mut vel in query.iter_mut() {
        if dir == Vec2::ZERO {
            vel.0 = Vec2::ZERO;
        } else {
            vel.0 = dir.normalize() * 30.;
        }
    }
}
