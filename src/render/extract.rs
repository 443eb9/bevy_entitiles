use bevy::{
    prelude::{
        Changed, Commands, Component, Entity, Handle, Image, Mat4, Or, Query, ResMut, Transform,
        UVec2, Vec2, Vec4, Without,
    },
    render::{render_resource::FilterMode, Extract},
};

use crate::tilemap::{Tile, TileType, Tilemap, WaitForTextureUsageChange};

use super::texture::{TilemapTextureArrayStorage, TilemapTextureDescriptor};

#[derive(Component)]
pub struct ExtractedTilemap {
    pub id: Entity,
    pub tile_type: TileType,
    pub size: UVec2,
    pub tile_size: UVec2,
    pub tile_render_size: Vec2,
    pub render_chunk_size: UVec2,
    pub texture: Handle<Image>,
    pub texture_desc: TilemapTextureDescriptor,
    pub filter_mode: FilterMode,
    pub transform: Mat4,
    pub flip: u32,
}

#[derive(Component, Debug)]
pub struct ExtractedTile {
    pub tilemap: Entity,
    pub render_chunk_index: usize,
    pub grid_index: UVec2,
    pub texture_index: u32,
    pub color: Vec4,
}

pub fn extract(
    mut commands: Commands,
    tilemaps_query: Extract<
        Query<(Entity, &Tilemap, &Transform), Without<WaitForTextureUsageChange>>,
    >,
    changed_tiles_query: Extract<Query<(Entity, &Tile), Or<(Changed<Tile>,)>>>,
    mut tilemap_texture_array_storage: ResMut<TilemapTextureArrayStorage>,
) {
    let mut extracted_tilemaps: Vec<(Entity, ExtractedTilemap)> = Vec::new();
    let mut extracted_tiles: Vec<(Entity, ExtractedTile)> = Vec::new();

    for (entity, tilemap, tilemap_transform) in tilemaps_query.iter() {
        tilemap_texture_array_storage.insert_texture(&tilemap.texture, &tilemap.texture_desc);

        extracted_tilemaps.push((
            entity,
            ExtractedTilemap {
                id: tilemap.id,
                tile_type: tilemap.tile_type.clone(),
                size: tilemap.size,
                tile_size: tilemap.tile_size,
                tile_render_size: tilemap.tile_render_size,
                render_chunk_size: tilemap.render_chunk_size,
                filter_mode: tilemap.filter_mode,
                texture: tilemap.texture.clone(),
                texture_desc: tilemap.texture_desc,
                transform: tilemap_transform.compute_matrix(),
                flip: tilemap.flip,
            },
        ));
    }

    for (entity, tile) in changed_tiles_query.iter() {
        extracted_tiles.push((
            entity,
            ExtractedTile {
                render_chunk_index: tile.render_chunk_index,
                tilemap: tile.tilemap_id,
                grid_index: tile.grid_index,
                texture_index: tile.texture_index,
                color: tile.color,
            },
        ));
    }

    commands.insert_or_spawn_batch(extracted_tiles);
    commands.insert_or_spawn_batch(extracted_tilemaps);
}
