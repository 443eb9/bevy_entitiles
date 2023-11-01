use bevy::{
    prelude::{
        Commands, Component, Entity, Handle, Image, Mat4, OrthographicProjection, Query, ResMut,
        Transform, UVec2,
    },
    render::{render_resource::FilterMode, Extract},
    utils::HashMap,
};

use crate::tilemap::{Tile, TileType, Tilemap};

use super::{
    chunk::TileData,
    cleanup::TilemapCleanUp,
    texture::{TilemapTextureArrayStorage, TilemapTextureDescriptor},
};

#[derive(Component)]
pub struct ExtractedTilemap {
    pub id: Entity,
    pub tile_type: TileType,
    pub size: UVec2,
    pub tile_size: UVec2,
    pub render_chunk_size: UVec2,
    pub texture: Handle<Image>,
    pub z_order: f32,
    pub filter_mode: FilterMode,
    pub transform: Mat4,
}

#[derive(Component)]
pub struct ExtractedTile {
    pub tilemap: Entity,
    pub render_chunk_index: usize,
    pub grid_index: UVec2,
    pub texture_index: u32,
}

pub fn extract(
    mut commands: Commands,
    tilemaps_query: Extract<Query<(Entity, &Tilemap, &Transform)>>,
    tiles_query: Extract<Query<(Entity, &Tile)>>,
    mut tilemap_texture_array_storage: ResMut<TilemapTextureArrayStorage>,
) {
    let mut extracted_tilemaps: Vec<(Entity, ExtractedTilemap)> = Vec::new();
    let mut extracted_tiles: Vec<(Entity, ExtractedTile)> = Vec::new();

    for (entity, tilemap, tilemap_transform) in tilemaps_query.iter() {
        extracted_tilemaps.push((
            entity,
            ExtractedTilemap {
                id: tilemap.id,
                tile_type: tilemap.tile_type.clone(),
                size: tilemap.size,
                tile_size: tilemap.tile_size,
                render_chunk_size: tilemap.render_chunk_size,
                filter_mode: tilemap.filter_mode,
                texture: tilemap.texture.clone_weak(),
                z_order: tilemap.z_order,
                transform: tilemap_transform.compute_matrix(),
            },
        ));

        tilemap_texture_array_storage.insert(
            &tilemap.texture,
            TilemapTextureDescriptor {
                tile_size: tilemap.tile_size,
                tile_count: tilemap.size.length_squared(),
                filter_mode: tilemap.filter_mode,
            },
        );
    }

    for (entity, tile) in tiles_query.iter() {
        extracted_tiles.push((
            entity,
            ExtractedTile {
                render_chunk_index: tile.render_chunk_index,
                tilemap: tile.tilemap_id,
                grid_index: tile.grid_index,
                texture_index: tile.texture_index,
            },
        ));
    }

    commands.insert_or_spawn_batch(extracted_tiles);
    commands.insert_or_spawn_batch(extracted_tilemaps);
}
