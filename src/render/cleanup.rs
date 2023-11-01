use bevy::prelude::{Commands, Component, Entity, Query, With};

use crate::tilemap::Tilemap;

#[derive(Component)]
pub struct TilemapCleanUp;

pub fn cleanup(
    mut commands: Commands,
    mut tilemaps: Query<(Entity, &mut Tilemap), With<TilemapCleanUp>>,
) {
    for (entity, mut tilemap) in tilemaps.iter_mut() {
        tilemap.render_chunks_to_update.clear();
        commands.entity(entity).remove::<TilemapCleanUp>();
    }
}
