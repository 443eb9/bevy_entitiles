use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        query::{Or, With},
        system::{Commands, ParallelCommands, Query},
    },
    math::IVec2,
};

use super::{map::TilemapStorage, tile::Tile};

#[derive(Component)]
pub struct DespawnMe;

#[derive(Component, Clone)]
pub struct DespawnedTilemap(pub Entity);

#[derive(Component, Clone)]
pub struct DespawnedTile {
    pub tilemap: Entity,
    pub chunk_index: IVec2,
    pub in_chunk_index: usize,
}

pub fn despawn_applier(
    commands: ParallelCommands,
    query: Query<Entity, Or<(With<DespawnedTilemap>, With<DespawnedTile>, With<DespawnMe>)>>,
) {
    query.par_iter().for_each(|e| {
        commands.command_scope(|mut c| {
            c.entity(e).despawn();
        });
    });
}

pub fn despawn_tilemap(
    mut commands: Commands,
    query: Query<Entity, (With<DespawnMe>, With<TilemapStorage>)>,
) {
    let mut despawned_tilemaps = Vec::new();

    query.for_each(|entity| {
        despawned_tilemaps.push(DespawnedTilemap(entity));
    });

    commands.spawn_batch(despawned_tilemaps);
}

pub fn despawn_tiles(mut commands: Commands, query: Query<&Tile, With<DespawnMe>>) {
    let mut despawned_tiles = Vec::new();

    query.for_each(|tile| {
        despawned_tiles.push(DespawnedTile {
            tilemap: tile.tilemap_id,
            chunk_index: tile.chunk_index,
            in_chunk_index: tile.in_chunk_index,
        });
    });

    commands.spawn_batch(despawned_tiles);
}
