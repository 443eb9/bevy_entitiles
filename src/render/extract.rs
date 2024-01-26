use bevy::{
    ecs::{event::EventReader, query::Without, system::Res},
    prelude::{Changed, Commands, Component, Entity, Query, Vec2, Vec4},
    render::Extract,
    time::Time,
    utils::EntityHashMap,
};

use crate::{
    math::CameraAabb2d,
    tilemap::{
        despawn::{DespawnedTile, DespawnedTilemap},
        map::{
            TilePivot, TileRenderSize, TilemapAnimations, TilemapAxisFlip,
            TilemapLayerOpacities, TilemapName, TilemapSlotSize, TilemapStorage, TilemapTexture,
            TilemapTransform, TilemapType,
        },
        tile::Tile,
    },
};

use super::{
    chunk::{ChunkUnload, UnloadRenderChunk},
    culling::{FrustumCulling, InvisibleTilemap},
};

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
    pub axis_flip: TilemapAxisFlip,
    pub texture: Option<TilemapTexture>,
    pub animations: Option<TilemapAnimations>,
    pub chunk_size: u32,
    pub time: f32,
}

pub type ExtractedTile = Tile;

pub type ExtractedView = CameraAabb2d;

pub fn extract_tilemaps(
    mut commands: Commands,
    tilemaps_query: Extract<
        Query<
            (
                Entity,
                &TilemapName,
                &TileRenderSize,
                &TilemapSlotSize,
                &TilemapType,
                &TilePivot,
                &TilemapLayerOpacities,
                &TilemapTransform,
                &TilemapAxisFlip,
                &TilemapStorage,
                Option<&TilemapTexture>,
                Option<&TilemapAnimations>,
            ),
            Without<InvisibleTilemap>,
        >,
    >,
    time: Extract<Res<Time>>,
) {
    commands.insert_or_spawn_batch(
        tilemaps_query
            .iter()
            .map(
                |(
                    entity,
                    name,
                    tile_render_size,
                    slot_size,
                    ty,
                    tile_pivot,
                    layer_opacities,
                    transform,
                    axis_flip,
                    storage,
                    texture,
                    animations,
                )| {
                    (
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
                            axis_flip: *axis_flip,
                            texture: texture.cloned(),
                            animations: animations.cloned(),
                            chunk_size: storage.storage.chunk_size,
                            time: time.elapsed_seconds(),
                        },
                    )
                },
            )
            .collect::<Vec<_>>(),
    );
}

pub fn extract_tiles(
    mut commands: Commands,
    tiles_query: Extract<Query<(Entity, &Tile), Changed<Tile>>>,
) {
    commands.insert_or_spawn_batch(
        tiles_query
            .iter()
            .map(|(entity, tile)| {
                (
                    entity,
                    ExtractedTile {
                        tilemap_id: tile.tilemap_id,
                        chunk_index: tile.chunk_index,
                        in_chunk_index: tile.in_chunk_index,
                        index: tile.index,
                        texture: tile.texture.clone(),
                        color: tile.color,
                    },
                )
            })
            .collect::<Vec<_>>(),
    );
}

pub fn extract_view(
    mut commands: Commands,
    cameras: Extract<Query<(Entity, &CameraAabb2d), Changed<CameraAabb2d>>>,
) {
    commands.insert_or_spawn_batch(
        cameras
            .iter()
            .map(|(e, aabb)| (e, *aabb))
            .collect::<Vec<_>>(),
    );
}

pub fn extract_unloaded_chunks(
    mut commands: Commands,
    mut chunk_unload: Extract<EventReader<ChunkUnload>>,
) {
    commands.insert_or_spawn_batch(chunk_unload.read().fold(
        EntityHashMap::<Entity, UnloadRenderChunk>::default(),
        |mut acc, elem| {
            acc.entry(elem.tilemap).or_default().0.push(elem.index);
            acc
        },
    ));
}

pub fn extract_resources(mut commands: Commands, frustum_culling: Extract<Res<FrustumCulling>>) {
    commands.insert_resource(FrustumCulling(frustum_culling.0));
}

pub fn extract_despawned_tilemaps(
    mut commands: Commands,
    tilemaps_query: Extract<Query<(Entity, &DespawnedTilemap)>>,
) {
    let mut despawned_tilemaps = Vec::new();

    tilemaps_query.for_each(|(entity, map)| {
        despawned_tilemaps.push((entity, map.clone()));
    });

    commands.insert_or_spawn_batch(despawned_tilemaps);
}

pub fn extract_despawned_tiles(
    mut commands: Commands,
    tiles_query: Extract<Query<(Entity, &DespawnedTile)>>,
) {
    let mut despawned_tiles = Vec::new();

    tiles_query.for_each(|(entity, tile)| {
        despawned_tiles.push((entity, tile.clone()));
    });

    commands.insert_or_spawn_batch(despawned_tiles);
}
