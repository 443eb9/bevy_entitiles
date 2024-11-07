use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        query::With,
        system::{Commands, ParallelCommands, Query},
    },
    math::IVec2,
};

use crate::tilemap::{map::TilemapStorage, tile::Tile};

/// Marks an tilemap/tile/physics_tilemap to be despawned on both CPU and GPU side.
#[derive(Component)]
pub struct DespawnMe;

/// Announced that a tilemap has been despawned for rendering.
#[derive(Component, Clone)]
pub struct DespawnedTilemap(pub Entity);

/// Announced that a tile has been despawned for rendering.
#[derive(Component, Clone)]
pub struct DespawnedTile {
    pub tilemap: Entity,
    pub chunk_index: IVec2,
    pub in_chunk_index: usize,
}

// This is kinda special. Theoretically, to despawn a tilemap, just insert `DespanwnMe`. But for those
// who want to reset render resources for tilemaps, they insert `DespawnTilemap`, and the actual entity
// won't be despawned.
pub fn despawn_component_remover(mut commands: Commands, query: Query<Entity>) {
    for entity in &query {
        commands
            .entity(entity)
            .remove::<DespawnedTilemap>()
            .remove::<DespawnedTile>();
    }
}

pub fn despawn_tilemap(
    mut commands: Commands,
    query: Query<Entity, (With<DespawnMe>, With<TilemapStorage>)>,
) {
    let mut despawned_tilemaps = Vec::new();

    query.iter().for_each(|entity| {
        despawned_tilemaps.push(DespawnedTilemap(entity));
        commands.entity(entity).despawn();
    });

    commands.spawn_batch(despawned_tilemaps);
}

pub fn despawn_tiles(mut commands: Commands, query: Query<(Entity, &Tile), With<DespawnMe>>) {
    let mut despawned_tiles = Vec::new();

    query.iter().for_each(|(entity, tile)| {
        despawned_tiles.push(DespawnedTile {
            tilemap: tile.tilemap_id,
            chunk_index: tile.chunk_index,
            in_chunk_index: tile.in_chunk_index,
        });
        commands.entity(entity).despawn();
    });

    commands.spawn_batch(despawned_tiles);
}

#[cfg(feature = "physics")]
pub fn despawn_physics_tilemaps(
    commands: ParallelCommands,
    query: Query<(Entity, &super::physics::PhysicsTilemap), With<DespawnMe>>,
) {
    query.par_iter().for_each(|(entity, physics_tilemap)| {
        commands.command_scope(|mut c| {
            physics_tilemap.storage.iter_some().for_each(|entity| {
                c.entity(*entity).despawn();
            });
            c.entity(entity).despawn();
        });
    });
}
