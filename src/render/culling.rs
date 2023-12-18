use bevy::{
    ecs::{entity::Entity, system::ParallelCommands},
    math::Vec3Swizzles,
    prelude::{Component, Query, ResMut},
    render::camera::OrthographicProjection,
    transform::components::Transform,
    window::Window,
};

use crate::{math::aabb::AabbBox2d, tilemap::map::Tilemap};

use super::{
    chunk::RenderChunkStorage,
    extract::{ExtractedTilemap, ExtractedView},
};

#[derive(Component)]
pub struct VisibleTilemap;

pub fn cull_tilemaps(
    commands: ParallelCommands,
    tilemaps: Query<(Entity, &Tilemap)>,
    cameras: Query<(&Transform, &OrthographicProjection)>,
    windows: Query<&Window>,
) {
    let Ok((trans, proj)) = cameras.get_single() else {
        return;
    };

    let Ok(window) = windows.get_single() else {
        return;
    };

    let camera_aabb = AabbBox2d::from_camera(&ExtractedView {
        width: window.width() / 2.,
        height: window.height() / 2.,
        scale: proj.scale,
        transform: trans.translation.xy(),
    });

    tilemaps.par_iter().for_each(|(entity, tilemap)| {
        commands.command_scope(|mut cmd| {
            if tilemap.aabb.is_intersected(&camera_aabb) {
                cmd.entity(entity).insert(VisibleTilemap);
            } else {
                cmd.entity(entity).remove::<VisibleTilemap>();
            }
        });
    });
}

pub fn cull_chunks(
    tilemaps: Query<&ExtractedTilemap>,
    mut render_chunk_storage: ResMut<RenderChunkStorage>,
    cameras: Query<&ExtractedView>,
) {
    let Ok(proj) = cameras.get_single() else {
        return;
    };

    for tilemap in tilemaps.iter() {
        let Some(chunks) = render_chunk_storage.get_chunks_mut(tilemap.id) else {
            break;
        };

        let camera_aabb = AabbBox2d::from_camera(proj);

        for chunk in chunks.iter_mut() {
            if let Some(c) = chunk {
                if c.aabb.is_intersected(&camera_aabb) {
                    c.visible = true;
                } else {
                    c.visible = false;
                }
            }
        }
    }
}
