use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        event::{Event, EventWriter},
        query::{Changed, Or},
        system::Query,
    },
    math::{IVec2, Vec2},
    reflect::Reflect,
    render::camera::OrthographicProjection,
    transform::components::Transform,
    utils::HashSet,
};

use crate::{math::CameraAabb2d, tilemap::map::TilemapStorage};

#[derive(Event, Debug, Clone, Copy, Reflect)]
pub enum CameraChunkUpdation {
    Entered(Entity, IVec2),
    Left(Entity, IVec2),
}

#[derive(Component, Debug, Clone, Reflect)]
pub struct CameraChunkUpdater {
    pub(crate) detect_scale: f32,
    pub(crate) update_scale: f32,
    pub(crate) last_updation: HashSet<IVec2>,
}

impl CameraChunkUpdater {
    /// These scales are used to scale the camera aabb for:
    /// - If a chunk is intersected with the detect aabb, then it must be visible.
    /// - If a chunk is intersected with the update aabb, then it must be loaded/generated
    ///   when there's some chunk invisible.
    /// 
    /// See the example `chunk_unloading` for further details.
    pub fn new(detect_scale: f32, update_scale: f32) -> Self {
        assert!(
            update_scale >= detect_scale,
            "update_scale must be >= detect_scale!"
        );

        Self {
            detect_scale,
            update_scale,
            last_updation: HashSet::new(),
        }
    }
}

pub fn camera_chunk_update(
    mut camera_query: Query<
        (&CameraAabb2d, &mut CameraChunkUpdater),
        Or<(Changed<OrthographicProjection>, Changed<Transform>)>,
    >,
    mut tilemaps_query: Query<(Entity, &TilemapStorage)>,
    mut updation_event: EventWriter<CameraChunkUpdation>,
) {
    camera_query.for_each_mut(|(cam_aabb, mut cam_updater)| {
        tilemaps_query.for_each_mut(|(entity, storage)| {
            // When the detect aabb is intersected with a invisible chunk,
            // all the chunks that are intercected with the update aabb must be visible.

            // Which means we need to first detect the chunks that are intersected with the detect aabb,
            // and if every one is visible, then do nothing else load/generate chunks that are intersected with the update aabb.

            let detect_aabb = cam_aabb
                .0
                .with_scale(Vec2::splat(cam_updater.detect_scale), Vec2::splat(0.5));

            let detected = storage
                .storage
                .reserved
                .iter()
                .filter_map(|(chunk_index, aabb)| {
                    if detect_aabb.is_intersected(*aabb) {
                        Some(*chunk_index)
                    } else {
                        None
                    }
                })
                .collect::<HashSet<_>>();

            if detected.is_subset(&cam_updater.last_updation) {
                return;
            }

            let update_aabb = cam_aabb
                .0
                .with_scale(Vec2::splat(cam_updater.update_scale), Vec2::splat(0.5));

            let mut cur_visible = HashSet::with_capacity(cam_updater.last_updation.len());

            storage
                .storage
                .reserved
                .iter()
                .for_each(|(chunk_index, aabb)| {
                    if update_aabb.is_intersected(*aabb) {
                        if !cam_updater.last_updation.contains(chunk_index) {
                            updation_event.send(CameraChunkUpdation::Entered(entity, *chunk_index));
                        }
                        cur_visible.insert(*chunk_index);
                    } else if cam_updater.last_updation.contains(chunk_index) {
                        updation_event.send(CameraChunkUpdation::Left(entity, *chunk_index));
                    }
                });

            cam_updater.last_updation = cur_visible;
        });
    });
}
