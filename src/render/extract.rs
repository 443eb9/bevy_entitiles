use bevy::{
    ecs::system::Res,
    math::{IVec4, UVec4, Vec3Swizzles},
    prelude::{
        Camera, Changed, Commands, Component, Entity, Or, OrthographicProjection, Query, Transform,
        UVec2, Vec2, Vec4,
    },
    render::Extract,
    time::Time,
    window::Window,
};

use crate::{
    math::aabb::AabbBox2d,
    tilemap::{
        map::Tilemap,
        tile::{AnimatedTile, Tile, TileType},
    },
    MAX_ANIM_COUNT,
};

use super::{buffer::TileAnimation, texture::TilemapTexture};

#[derive(Component, Debug)]
pub struct ExtractedTilemap {
    pub id: Entity,
    pub tile_type: TileType,
    pub ext_dir: Vec2,
    pub size: UVec2,
    pub tile_render_size: Vec2,
    pub tile_slot_size: Vec2,
    pub pivot: Vec2,
    pub render_chunk_size: u32,
    pub texture: Option<TilemapTexture>,
    pub translation: Vec2,
    pub aabb: AabbBox2d,
    pub z_index: i32,
    pub anim_seqs: [TileAnimation; MAX_ANIM_COUNT],
    pub layer_opacities: Vec4,
    pub time: f32,
}

#[derive(Component)]
pub struct ExtractedTile {
    pub tilemap: Entity,
    pub render_chunk_index: usize,
    pub index: UVec2,
    pub texture_indices: IVec4,
    pub color: Vec4,
    pub anim: Option<AnimatedTile>,
    pub flip: UVec4,
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
    time: Extract<Res<Time>>,
) {
    let mut extracted_tilemaps = vec![];
    for (entity, tilemap) in tilemaps_query.iter() {
        extracted_tilemaps.push((
            entity,
            ExtractedTilemap {
                id: tilemap.id,
                tile_type: tilemap.tile_type,
                ext_dir: tilemap.ext_dir,
                size: tilemap.size,
                tile_render_size: tilemap.tile_render_size,
                tile_slot_size: tilemap.tile_slot_size,
                render_chunk_size: tilemap.render_chunk_size,
                pivot: tilemap.pivot,
                texture: tilemap.texture.clone(),
                translation: tilemap.translation,
                aabb: tilemap.aabb,
                z_index: tilemap.z_index,
                anim_seqs: tilemap.anim_seqs,
                layer_opacities: tilemap.layer_opacities,
                time: time.elapsed_seconds(),
            },
        ));
    }

    commands.insert_or_spawn_batch(extracted_tilemaps);
}

pub fn extract_tiles(
    mut commands: Commands,
    changed_tiles_query: Extract<Query<(Entity, &Tile, Option<&AnimatedTile>), Changed<Tile>>>,
) {
    let mut extracted_tiles: Vec<(Entity, ExtractedTile)> = vec![];
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
                texture_indices: tile.texture_indices,
                color: tile.color,
                anim,
                flip: tile.flip,
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
