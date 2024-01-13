use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        system::{ParallelCommands, Res, Resource},
    },
    prelude::{Query, ResMut},
};

use crate::{math::CameraAabb2d, tilemap::map::TilemapAabbs};

use super::{
    chunk::RenderChunkStorage,
    extract::{ExtractedTilemap, ExtractedView},
};

#[derive(Component)]
pub struct InvisibleTilemap;

#[derive(Resource)]
pub struct FrustumCulling(pub bool);

impl Default for FrustumCulling {
    fn default() -> Self {
        Self(true)
    }
}

pub fn cull_tilemaps(
    commands: ParallelCommands,
    tilemaps: Query<(Entity, &TilemapAabbs)>,
    cameras: Query<&CameraAabb2d>,
) {
    cameras.par_iter().for_each(|camera| {
        tilemaps.par_iter().for_each(|(entity, aabbs)| {
            commands.command_scope(|mut c| {
                if !aabbs.world_aabb.is_intersected(camera.0) {
                    c.entity(entity).insert(InvisibleTilemap);
                } else {
                    c.entity(entity).remove::<InvisibleTilemap>();
                }
            });
        });
    });
}

pub fn cull_chunks(
    tilemaps: Query<&ExtractedTilemap>,
    mut render_chunk_storage: ResMut<RenderChunkStorage>,
    cameras: Query<&ExtractedView>,
    culling: Res<FrustumCulling>,
) {
    if !culling.0 {
        return;
    }

    cameras.for_each(|cam_aabb| {
        tilemaps.for_each(|tilemap| {
            let Some(chunks) = render_chunk_storage.get_chunks_mut(tilemap.id) else {
                return;
            };

            chunks.values_mut().for_each(|c| {
                if c.aabb.is_intersected(cam_aabb.0) {
                    c.visible = true;
                } else {
                    c.visible = false;
                }
            });
        });
    });
}
