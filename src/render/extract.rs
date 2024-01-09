use bevy::{
    ecs::{event::EventReader, system::Res},
    math::{IVec2, Vec3Swizzles},
    prelude::{
        Camera, Changed, Commands, Component, Entity, Or, OrthographicProjection, Query, Transform,
        UVec2, Vec2, Vec4,
    },
    render::Extract,
    time::Time,
    utils::EntityHashMap,
};

use crate::tilemap::{
    map::{
        TilePivot, TileRenderSize, TilemapAnimations, TilemapLayerOpacities, TilemapName,
        TilemapSlotSize, TilemapStorage, TilemapTexture, TilemapTransform, TilemapType,
    },
    tile::{Tile, TileTexture},
};

use super::chunk::{ChunkUnload, UnloadRenderChunk};

#[derive(Component, Debug)]
pub struct ExtractedTilemap {
    pub id: Entity,
    pub name: String,
    pub tile_render_size: Vec2,
    pub slot_size: Vec2,
    pub ty: TilemapType,
    pub tile_pivot: Vec2,
    pub layer_opacities: Vec4,
    pub transform: TilemapTransform,
    pub texture: Option<TilemapTexture>,
    pub animations: Option<TilemapAnimations>,
    pub chunk_size: u32,
    pub time: f32,
}

#[derive(Component, Debug)]
pub struct ExtractedTile {
    pub tilemap: Entity,
    pub chunk_index: IVec2,
    pub in_chunk_index: UVec2,
    pub index: IVec2,
    pub texture: TileTexture,
    pub color: Vec4,
}

#[derive(Component, Debug)]
pub struct ExtractedView {
    pub min: Vec2,
    pub max: Vec2,
    pub transform: Vec2,
}

pub fn extract_tilemaps(
    mut commands: Commands,
    tilemaps_query: Extract<
        Query<(
            Entity,
            &TilemapName,
            &TileRenderSize,
            &TilemapSlotSize,
            &TilemapType,
            &TilePivot,
            &TilemapLayerOpacities,
            &TilemapTransform,
            &TilemapStorage,
            Option<&TilemapTexture>,
            Option<&TilemapAnimations>,
        )>,
    >,
    time: Extract<Res<Time>>,
) {
    let mut extracted_tilemaps = vec![];
    for (
        entity,
        name,
        tile_render_size,
        slot_size,
        ty,
        tile_pivot,
        layer_opacities,
        transform,
        storage,
        texture,
        animations,
    ) in tilemaps_query.iter()
    {
        extracted_tilemaps.push((
            entity,
            ExtractedTilemap {
                id: entity,
                name: name.0.clone(),
                tile_render_size: tile_render_size.0,
                slot_size: slot_size.0,
                ty: *ty,
                tile_pivot: tile_pivot.0,
                layer_opacities: layer_opacities.0,
                transform: *transform,
                texture: texture.cloned(),
                animations: animations.cloned(),
                chunk_size: storage.storage.chunk_size,
                time: time.elapsed_seconds(),
            },
        ));
    }

    commands.insert_or_spawn_batch(extracted_tilemaps);
}

pub fn extract_tiles(
    mut commands: Commands,
    changed_tiles_query: Extract<Query<(Entity, &Tile), Changed<Tile>>>,
) {
    let mut extracted_tiles: Vec<(Entity, ExtractedTile)> = vec![];
    for (entity, tile) in changed_tiles_query.iter() {
        extracted_tiles.push((
            entity,
            ExtractedTile {
                tilemap: tile.tilemap_id,
                chunk_index: tile.chunk_index,
                in_chunk_index: tile.in_chunk_index,
                index: tile.index,
                texture: tile.texture.clone(),
                color: tile.color,
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
) {
    let mut extracted_cameras = vec![];
    for (entity, projection, _, transform) in cameras.iter() {
        extracted_cameras.push((
            entity,
            ExtractedView {
                min: projection.area.min,
                max: projection.area.max,
                transform: transform.translation.xy(),
            },
        ));
    }
    commands.insert_or_spawn_batch(extracted_cameras);
}

pub fn extract_unloaded_chunks(mut commands: Commands, mut chunk_unload: Extract<EventReader<ChunkUnload>>) {
    commands.insert_or_spawn_batch(chunk_unload.read().fold(
        EntityHashMap::<Entity, UnloadRenderChunk>::default(),
        |mut acc, elem| {
            acc.entry(elem.tilemap).or_default().0.push(elem.index);
            acc
        },
    ));
}
