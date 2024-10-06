use bevy::{
    ecs::entity::Entity,
    math::IVec2,
    prelude::{Query, Res, ResMut},
};

use crate::{
    render::{
        chunk::{RenderChunkSort, UnloadRenderChunk},
        extract::{ExtractedTile, TilemapInstances},
        material::TilemapMaterial,
        RenderChunkStorage,
    },
    tilemap::despawn::{DespawnedTile, DespawnedTilemap},
};

pub fn prepare_tiles<M: TilemapMaterial>(
    extracted_tiles: Query<&mut ExtractedTile>,
    mut render_chunks: ResMut<RenderChunkStorage>,
    tilemap_instances: Res<TilemapInstances>,
) {
    extracted_tiles.iter().for_each(|tile| {
        let Some(tilemap) = tilemap_instances.get(&tile.tilemap_id) else {
            return;
        };

        let chunks = render_chunks.get_or_insert_chunks(tile.tilemap_id);
        chunks.try_add_chunk(tile.chunk_index, tilemap);
        chunks.set_tile(tile);
    });
}

pub fn prepare_unloaded_chunks<M: TilemapMaterial>(
    mut render_chunks: ResMut<RenderChunkStorage>,
    extracted_tilemaps: Query<(Entity, &UnloadRenderChunk)>,
) {
    extracted_tilemaps.iter().for_each(|(entity, unloaded)| {
        unloaded.0.iter().for_each(|c| {
            render_chunks.get_or_insert_chunks(entity).remove_chunk(*c);
        });
    });
}

pub fn prepare_despawned_tilemaps<M: TilemapMaterial>(
    mut render_chunks: ResMut<RenderChunkStorage>,
    // mut storage_buffers: ResMut<TilemapAnimationBuffer>,
    mut tilemap_instances: ResMut<TilemapInstances>,
    tilemaps_query: Query<&DespawnedTilemap>,
) {
    tilemaps_query.iter().for_each(|map| {
        render_chunks.remove_tilemap(map.0);
        // storage_buffers.remove(map.0);
        tilemap_instances.remove(&map.0);
    });
}

pub fn prepare_despawned_tiles<M: TilemapMaterial>(
    mut render_chunks: ResMut<RenderChunkStorage>,
    tiles_query: Query<&DespawnedTile>,
) {
    tiles_query.iter().for_each(|tile| {
        render_chunks
            .get_or_insert_chunks(tile.tilemap)
            .remove_tile(tile.chunk_index, tile.in_chunk_index);
    });
}

pub fn sort_chunks<M: TilemapMaterial>(
    mut render_chunks: ResMut<RenderChunkStorage>,
    sort_config: Res<RenderChunkSort>,
) {
    let cfg = sort_config.into_inner();
    if matches!(cfg, RenderChunkSort::None) {
        return;
    }

    render_chunks.sort(match cfg {
        RenderChunkSort::XThenY => {
            |lhs: IVec2, rhs: IVec2| lhs.x.cmp(&rhs.x).then_with(|| lhs.y.cmp(&rhs.y))
        }
        RenderChunkSort::XReverseThenY => {
            |lhs: IVec2, rhs: IVec2| rhs.x.cmp(&lhs.x).then_with(|| lhs.y.cmp(&rhs.y))
        }
        RenderChunkSort::XThenYReverse => {
            |lhs: IVec2, rhs: IVec2| lhs.x.cmp(&rhs.x).then_with(|| rhs.y.cmp(&lhs.y))
        }
        RenderChunkSort::XReverseThenYReverse => {
            |lhs: IVec2, rhs: IVec2| rhs.x.cmp(&lhs.x).then_with(|| rhs.y.cmp(&lhs.y))
        }
        RenderChunkSort::YThenX => {
            |lhs: IVec2, rhs: IVec2| lhs.y.cmp(&rhs.y).then_with(|| lhs.x.cmp(&rhs.x))
        }
        RenderChunkSort::YReverseThenX => {
            |lhs: IVec2, rhs: IVec2| rhs.y.cmp(&lhs.y).then_with(|| lhs.x.cmp(&rhs.x))
        }
        RenderChunkSort::YThenXReverse => {
            |lhs: IVec2, rhs: IVec2| lhs.y.cmp(&rhs.y).then_with(|| rhs.x.cmp(&lhs.x))
        }
        RenderChunkSort::YReverseThenXReverse => {
            |lhs: IVec2, rhs: IVec2| rhs.y.cmp(&lhs.y).then_with(|| rhs.x.cmp(&lhs.x))
        }
        RenderChunkSort::None => unreachable!(),
    });
}
