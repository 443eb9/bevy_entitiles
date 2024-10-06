use bevy::{
    app::{App, Plugin, Update},
    ecs::{
        component::Component,
        entity::Entity,
        query::{Added, Changed, Or},
        system::{Commands, Query},
    },
    math::{IRect, IVec2, Rect, Vec3Swizzles},
    prelude::UVec2,
    reflect::Reflect,
    render::camera::{Camera, OrthographicProjection},
    transform::components::Transform,
};

use crate::math::ext::RectTransformation;

pub mod ext;

pub struct EntiTilesMathPlugin;

impl Plugin for EntiTilesMathPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (camera_aabb_adder, camera_aabb_updater));

        app.register_type::<GridRect>()
            .register_type::<CameraAabb2d>();
    }
}

#[derive(Component, Default, Debug, Clone, Copy, Reflect)]
pub struct CameraAabb2d(pub Rect);

pub fn camera_aabb_adder(mut commands: Commands, cameras_query: Query<Entity, Added<Camera>>) {
    cameras_query.iter().for_each(|e| {
        commands.entity(e).insert(CameraAabb2d::default());
    });
}

pub fn camera_aabb_updater(
    mut commands: Commands,
    mut cameras_query: Query<
        (Entity, &OrthographicProjection, &Transform),
        Or<(
            Changed<OrthographicProjection>,
            Changed<Transform>,
            Added<OrthographicProjection>,
        )>,
    >,
    #[cfg(feature = "debug")] camera_aabb_scale: bevy::ecs::system::Res<
        crate::debug::CameraAabbScale,
    >,
) {
    cameras_query.iter_mut().for_each(|(entity, proj, trans)| {
        #[cfg(feature = "debug")]
        commands.entity(entity).insert(CameraAabb2d(
            Rect {
                min: proj.area.min,
                max: proj.area.max,
            }
            .with_translation(trans.translation.xy())
            .with_scale(camera_aabb_scale.0, bevy::math::Vec2::splat(0.5)),
        ));
        #[cfg(not(feature = "debug"))]
        commands.entity(entity).insert(CameraAabb2d(
            Rect {
                min: proj.area.min,
                max: proj.area.max,
            }
            .with_translation(trans.translation.xy()),
        ));
    });
}

#[derive(Debug, Clone, Copy, Reflect)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub struct GridRect {
    pub origin: IVec2,
    pub extent: UVec2,
    pub dest: IVec2,
}

impl Default for GridRect {
    fn default() -> Self {
        Self::EMPTY
    }
}

impl GridRect {
    pub const EMPTY: Self = Self {
        origin: IVec2::MAX,
        extent: UVec2::ZERO,
        dest: IVec2::MIN,
    };

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
    pub fn from_min_max(min: IVec2, max: IVec2) -> Self {
        Self {
            origin: min,
            extent: (max - min).as_uvec2() + 1,
            dest: max,
        }
    }

    #[inline]
    pub fn size(&self) -> usize {
        (self.extent.x * self.extent.y) as usize
    }

    #[inline]
    pub fn contains(&self, p: IVec2) -> bool {
        (self.origin.cmple(p) & self.dest.cmpge(p)).all()
    }

    #[inline]
    pub fn union_point(&self, other: IVec2) -> Self {
        let origin = self.origin.min(other);
        let dest = self.dest.max(other);
        Self::from_min_max(origin, dest)
    }

    /// Convert the area into rect, but with exclusive max value.
    #[inline]
    pub fn into_rect(&self) -> IRect {
        IRect {
            min: self.origin,
            max: self.dest,
        }
    }

    /// Convert the area into rect, and with inclusive max value.
    #[inline]
    pub fn into_rect_inclusive(&self) -> IRect {
        IRect {
            min: self.origin,
            max: self.dest + 1,
        }
    }

    /// Convert the rect into area, but with exclusive max value.
    #[inline]
    pub fn from_rect(rect: IRect) -> Self {
        Self {
            origin: rect.min,
            extent: rect.size().as_uvec2(),
            dest: rect.max,
        }
    }

    /// Convert the rect into area, and with inclusive max value.
    #[inline]
    pub fn from_rect_inclusive(rect: IRect) -> Self {
        Self {
            origin: rect.min,
            extent: rect.size().as_uvec2() + 1,
            dest: rect.max + 1,
        }
    }
}
