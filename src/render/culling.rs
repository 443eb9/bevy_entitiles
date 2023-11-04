use bevy::prelude::{Changed, Commands, Component, Or, Query, Res, ResMut, Resource, Transform};

use crate::math::geometry::AabbBox2d;

use super::{chunk::RenderChunkStorage, extract::ExtractedTilemap};

#[derive(Component)]
pub struct Invisible;

#[derive(Resource)]
pub struct CameraVisibleBox(AabbBox2d);

pub fn cull(
    mut commands: Commands,
    mut tilemaps: Query<&mut ExtractedTilemap, Or<(Changed<Transform>,)>>,
    render_chunk_storage: Res<RenderChunkStorage>,
) {
}
