use bevy::{
    app::{App, Plugin, Update},
    ecs::{
        component::Component,
        entity::Entity,
        query::Added,
        system::{Commands, Query},
    },
    math::IVec2,
    prelude::UVec2,
    reflect::Reflect,
    render::camera::{Camera, OrthographicProjection},
    transform::components::Transform,
};

use self::aabb::Aabb2d;

pub mod aabb;
pub mod extension;

pub struct EntiTilesMathPlugin;

impl Plugin for EntiTilesMathPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (camera_aabb_adder, camera_aabb_updater));

        app.register_type::<Aabb2d>()
            .register_type::<TileArea>()
            .register_type::<CameraAabb2d>();
    }
}

#[derive(Component, Debug, Clone, Copy, Reflect)]
pub struct CameraAabb2d(pub Aabb2d);

pub fn camera_aabb_adder(mut commands: Commands, cameras_query: Query<Entity, Added<Camera>>) {
    cameras_query.for_each(|e| {
        commands.entity(e).insert(CameraAabb2d(Aabb2d::default()));
    });
}

pub fn camera_aabb_updater(
    mut cameras_query: Query<(&OrthographicProjection, &Transform, &mut CameraAabb2d)>,
) {
}

#[derive(Debug, Clone, Copy, Reflect)]
pub struct TileArea {
    pub origin: IVec2,
    pub extent: UVec2,
    pub dest: IVec2,
}

impl TileArea {
    /// Define a new fill area without checking if the area is out of the tilemap.
    #[inline]
    pub fn new(origin: IVec2, extent: UVec2) -> Self {
        Self {
            origin,
            extent,
            dest: origin + extent.as_ivec2() - 1,
        }
    }

    #[inline]
    pub fn size(&self) -> usize {
        (self.extent.x * self.extent.y) as usize
    }
}
