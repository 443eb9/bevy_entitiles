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
    math::Vec3,
    prelude::{App, AssetServer, Camera2dBundle, Commands, Res, Startup, UVec2, Update, Vec2},
    render::{
        color::Color,
        mesh::{shape::Circle, Mesh},
        render_resource::FilterMode,
    },
    sprite::{ColorMaterial, ColorMesh2dBundle},
    time::Time,
    transform::components::Transform,
    DefaultPlugins,
};
use bevy_entitiles::{
    debug::camera_movement::camera_control,
    math::FillArea,
    render::texture::TilemapTextureDescriptor,
    tilemap::{physics::TileCollision, TileBuilder, TileType, TilemapBuilder},
    EntiTilesPlugin,
};
use bevy_xpbd_2d::components::{Collider, RigidBody};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EntiTilesPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, (camera_control, collision_events))
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

    let (tilemap_entity, mut tilemap) = TilemapBuilder::new(
        TileType::IsometricDiamond,
        UVec2 { x: 20, y: 10 },
        Vec2 { x: 64.0, y: 32.0 },
    )
    .with_texture(
        assets_server.load("test/test_isometric.png"),
        TilemapTextureDescriptor {
            tile_count: UVec2 { x: 1, y: 2 },
            tile_size: UVec2 { x: 32, y: 16 },
            filter_mode: FilterMode::Nearest,
        },
    )
    .build(&mut commands);

    tilemap.fill_rect(
        &mut commands,
        FillArea::full(&tilemap),
        &TileBuilder::new(UVec2::ZERO, 0),
    );

    tilemap.set_physics_tile_xpbd(&mut commands, UVec2 { x: 19, y: 9 }, None, true);

    tilemap.fill_physics_tile_xpbd(
        &mut commands,
        FillArea::new(UVec2::ZERO, Some(UVec2 { x: 5, y: 5 }), &tilemap),
        Some(0.8),
        false,
    );

    commands.entity(tilemap_entity).insert(tilemap);

    let (tilemap_entity, mut tilemap) = TilemapBuilder::new(
        TileType::Square,
        UVec2 { x: 20, y: 10 },
        Vec2 { x: 32.0, y: 32.0 },
    )
    .with_translation(Vec2 { x: 500., y: -100. })
    .with_texture(
        assets_server.load("test/test_square.png"),
        TilemapTextureDescriptor {
            tile_count: UVec2 { x: 2, y: 2 },
            tile_size: UVec2 { x: 16, y: 16 },
            filter_mode: FilterMode::Nearest,
        },
    )
    .build(&mut commands);

    tilemap.fill_rect(
        &mut commands,
        FillArea::full(&tilemap),
        &TileBuilder::new(UVec2::ZERO, 0),
    );

    tilemap.set_physics_tile_xpbd(&mut commands, UVec2 { x: 19, y: 9 }, None, true);

    tilemap.fill_physics_tile_xpbd(
        &mut commands,
        FillArea::new(UVec2::ZERO, Some(UVec2 { x: 5, y: 5 }), &tilemap),
        Some(0.8),
        false,
    );

    commands.entity(tilemap_entity).insert(tilemap);

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
        Character,
    ));
}

fn collision_events(mut collision: EventReader<TileCollision>) {
    for c in collision.read() {
        println!("Collision: {:?}", c.tile_snapshot);
    }
}

#[derive(Component)]
pub struct Character;

pub fn character_move(
    input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Character>>,
    time: Res<Time>,
) {
    let step = 30. * time.delta_seconds();
    for mut trans in query.iter_mut() {
        if input.pressed(KeyCode::Up) {
            trans.translation.y += step;
        }
        if input.pressed(KeyCode::Down) {
            trans.translation.y -= step;
        }
        if input.pressed(KeyCode::Left) {
            trans.translation.x -= step;
        }
        if input.pressed(KeyCode::Right) {
            trans.translation.x += step;
        }
    }
}
