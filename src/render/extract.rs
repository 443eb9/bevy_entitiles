use bevy::{
    math::Vec3Swizzles,
    prelude::{
        Camera, Changed, Commands, Component, Entity, Or, OrthographicProjection, Query, ResMut,
        Transform, UVec2, Vec2, Vec4,
    },
    render::{render_resource::FilterMode, Extract},
    window::Window,
};

use crate::{
    math::aabb::AabbBox2d, tilemap::{tile::{TileAnimation, Tile, TileType, TileTexture}, map::Tilemap},
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
    pub translation: Vec2,
    pub flip: u32,
    pub aabb: AabbBox2d,
    pub z_order: u32,
}

#[derive(Component)]
pub struct ExtractedTile {
    pub tilemap: Entity,
    pub render_chunk_index: usize,
    pub index: UVec2,
    pub texture_index: u32,
    pub color: Vec4,
    pub anim: Option<TileAnimation>,
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
    tilemaps_query: Extract<Query<(Entity, &Tilemap)>>,
    mut tilemap_texture_array_storage: ResMut<TilemapTextureArrayStorage>,
) {
    let mut extracted_tilemaps: Vec<(Entity, ExtractedTilemap)> = Vec::new();
    for (entity, tilemap) in tilemaps_query.iter() {
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
                translation: tilemap.translation,
                flip: tilemap.flip,
                aabb: tilemap.aabb.clone(),
                z_order: tilemap.z_order,
            },
        ));
    }

    commands.insert_or_spawn_batch(extracted_tilemaps);
}

pub fn extract_tiles(
    mut commands: Commands,
    changed_tiles_query: Extract<
        Query<(Entity, &Tile, Option<&TileAnimation>), Or<(Changed<Tile>, &TileAnimation)>>,
    >,
) {
    let mut extracted_tiles: Vec<(Entity, ExtractedTile)> = Vec::new();
    for (entity, tile, anim) in changed_tiles_query.iter() {
        let anim = if let Some(a) = anim {
            Some(a.clone())
        } else {
            None
        };
        extracted_tiles.push((
            entity,
            ExtractedTile {
                render_chunk_index: tile.render_chunk_index,
                tilemap: tile.tilemap_id,
                index: tile.index,
                texture_index: tile.texture_index,
                color: tile.color,
                anim,
            },
        ));
    }
    commands.insert_or_spawn_batch(extracted_tiles);
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
    for (entity, projection, _, transform) in cameras.iter() {
        extracted_cameras.push((
            entity,
            ExtractedView {
                width: window.width() / 2.,
                height: window.height() / 2.,
                scale: projection.scale,
                transform: transform.translation.xy(),
            },
        ));
    }
    commands.insert_or_spawn_batch(extracted_cameras);
}
