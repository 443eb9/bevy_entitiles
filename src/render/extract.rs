use bevy::{
    math::Vec3Swizzles,
    prelude::{
        Camera, Changed, Commands, Component, Entity, Mat4, Or, OrthographicProjection, Query,
        ResMut, Transform, UVec2, Vec2, Vec4, Without,
    },
    render::{render_resource::FilterMode, Extract},
    window::Window,
};

use crate::{
    math::aabb::AabbBox2d,
    tilemap::{Tile, TileTexture, TileType, Tilemap, WaitForTextureUsageChange},
};

use super::texture::TilemapTextureArrayStorage;

#[derive(Component)]
pub struct ExtractedTilemap {
    pub id: Entity,
    pub tile_type: TileType,
    pub size: UVec2,
    pub tile_size: UVec2,
    pub tile_render_size: Vec2,
    pub render_chunk_size: u32,
    pub texture: Option<TileTexture>,
    pub filter_mode: FilterMode,
    pub transfrom: Transform,
    pub transform_matrix: Mat4,
    pub flip: u32,
    pub aabb: AabbBox2d,
}

impl ExtractedTilemap {
    pub fn get_center_in_world(&self) -> Vec2 {
        self.aabb.center()
    }
}

#[derive(Component, Debug)]
pub struct ExtractedTile {
    pub tilemap: Entity,
    pub render_chunk_index: usize,
    pub grid_index: UVec2,
    pub texture_index: u32,
    pub color: Vec4,
}

#[derive(Component)]
pub struct ExtractedView {
    pub width: f32,
    pub height: f32,
    pub scale: f32,
    pub transform: Vec2,
}

pub fn extract_tilemaps(
    mut commands: Commands,
    tilemaps_query: Extract<
        Query<(Entity, &Tilemap, &Transform), Without<WaitForTextureUsageChange>>,
    >,
    changed_tiles_query: Extract<Query<(Entity, &Tile), Or<(Changed<Tile>,)>>>,
    mut tilemap_texture_array_storage: ResMut<TilemapTextureArrayStorage>,
) {
    let mut extracted_tilemaps: Vec<(Entity, ExtractedTilemap)> = Vec::new();
    for (entity, tilemap, tilemap_transform) in tilemaps_query.iter() {
        if let Some(texture) = &tilemap.texture {
            tilemap_texture_array_storage.insert_texture(texture.clone());
        }

        extracted_tilemaps.push((
            entity,
            ExtractedTilemap {
                id: tilemap.id,
                tile_type: tilemap.tile_type,
                size: tilemap.size,
                tile_size: tilemap.tile_size,
                tile_render_size: tilemap.tile_render_size,
                render_chunk_size: tilemap.render_chunk_size,
                filter_mode: tilemap.filter_mode,
                texture: tilemap.texture.clone(),
                transfrom: *tilemap_transform,
                transform_matrix: tilemap_transform.compute_matrix(),
                flip: tilemap.flip,
                aabb: tilemap.aabb.clone(),
            },
        ));
    }

    let mut extracted_tiles: Vec<(Entity, ExtractedTile)> = Vec::new();
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

pub fn extract_view(
    mut commands: Commands,
    cameras: Extract<
        Query<
            (Entity, &OrthographicProjection, &Camera, &Transform),
            Or<(Changed<Transform>, Changed<OrthographicProjection>)>,
        >,
    >,
    windows: Extract<Query<&Window>>,
) {
    let Ok(window) = windows.get_single() else {
        return;
    };

    let mut extracted_cameras = vec![];
    for (entity, projection, camera, transform) in cameras.iter() {
        extracted_cameras.push((
            entity,
            ExtractedView {
                width: window.width(),
                height: window.height(),
                scale: projection.scale,
                transform: transform.translation.xy(),
            },
        ));
    }
    commands.insert_or_spawn_batch(extracted_cameras);
}
