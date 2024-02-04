use bevy::{
    ecs::{entity::Entity, query::With},
    prelude::{Commands, Query, Res, ResMut},
    render::renderer::{RenderDevice, RenderQueue},
    time::Time,
};

use crate::tilemap::despawn::{DespawnedTile, DespawnedTilemap};

use super::{
    binding::TilemapBindGroups,
    buffer::{
        PerTilemapBuffersStorage, TilemapStorageBuffers, TilemapUniformBuffer, UniformBuffer,
    },
    chunk::{TilemapRenderChunk, UnloadRenderChunk},
    extract::{ExtractedTile, TilemapInstance},
    pipeline::EntiTilesPipeline,
    resources::TilemapInstances,
    texture::TilemapTexturesStorage,
    RenderChunkStorage,
};

pub fn prepare_tilemaps(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    extracted_tilemaps: Query<Entity, With<TilemapInstance>>,
    mut render_chunks: ResMut<RenderChunkStorage>,
    mut uniform_buffers: ResMut<TilemapUniformBuffer>,
    mut storage_buffers: ResMut<TilemapStorageBuffers>,
    mut textures_storage: ResMut<TilemapTexturesStorage>,
    entitiles_pipeline: Res<EntiTilesPipeline>,
    mut bind_groups: ResMut<TilemapBindGroups>,
    time: Res<Time>,
    tilemap_instances: Res<TilemapInstances>,
) {
    uniform_buffers.clear();
    storage_buffers.clear();

    extracted_tilemaps
        .iter()
        .filter_map(|tilemap| tilemap_instances.get(tilemap))
        .for_each(|tilemap| {
            commands
                .entity(tilemap.id)
                .insert(uniform_buffers.insert(&(tilemap, time.elapsed_seconds())));

            render_chunks.prepare_chunks(tilemap, &render_device);

            if let Some(texture) = tilemap.texture.as_ref() {
                storage_buffers
                    .get_or_insert_buffer(tilemap.id)
                    .extend(&tilemap.animations.as_ref().unwrap().0);

                if !textures_storage.contains(&texture.texture) {
                    textures_storage.insert(texture.clone_weak(), texture.desc());
                }
            }
        });

    #[cfg(not(feature = "atlas"))]
    textures_storage.prepare_textures(&render_device);
    uniform_buffers.write(&render_device, &render_queue);
    storage_buffers.write(&render_device, &render_queue);

    bind_groups.bind_uniform_buffers(&render_device, &mut uniform_buffers, &entitiles_pipeline);
    bind_groups.bind_storage_buffers(&render_device, &mut storage_buffers, &entitiles_pipeline);
}

pub fn prepare_tiles(
    extracted_tiles: Query<&mut ExtractedTile>,
    mut render_chunks: ResMut<RenderChunkStorage>,
    tilemap_instances: Res<TilemapInstances>,
) {
    extracted_tiles.for_each(|tile| {
        let Some(tilemap) = tilemap_instances.get(tile.tilemap_id) else {
            return;
        };

        let chunks = render_chunks.value.entry(tile.tilemap_id).or_default();

        let chunk = chunks
            .entry(tile.chunk_index)
            .or_insert_with(|| TilemapRenderChunk::from_index(tile.chunk_index, tilemap));

        chunk.set_tile(tile.in_chunk_index, Some(tile));
    });
}

pub fn prepare_unloaded_chunks(
    mut render_chunks: ResMut<RenderChunkStorage>,
    extracted_tilemaps: Query<(Entity, &UnloadRenderChunk)>,
) {
    extracted_tilemaps.for_each(|(entity, unloaded)| {
        unloaded.0.iter().for_each(|c| {
            render_chunks.remove_chunk(entity, *c);
        });
    });
}

pub fn prepare_despawned_tilemaps(
    mut render_chunks: ResMut<RenderChunkStorage>,
    mut storage_buffers: ResMut<TilemapStorageBuffers>,
    mut tilemap_instaces: ResMut<TilemapInstances>,
    tilemaps_query: Query<&DespawnedTilemap>,
) {
    tilemaps_query.for_each(|map| {
        render_chunks.remove_tilemap(map.0);
        storage_buffers.remove(map.0);
        tilemap_instaces.remove(map.0);
    });
}

pub fn prepare_despawned_tiles(
    mut render_chunks: ResMut<RenderChunkStorage>,
    tiles_query: Query<&DespawnedTile>,
) {
    tiles_query.for_each(|tile| {
        if let Some(chunk) = render_chunks
            .get_chunks_mut(tile.tilemap)
            .and_then(|chunks| chunks.get_mut(&tile.chunk_index))
        {
            chunk.set_tile(tile.in_chunk_index, None);
        }
    });
}
