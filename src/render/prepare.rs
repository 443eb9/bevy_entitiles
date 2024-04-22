use bevy::{
    ecs::{entity::Entity, query::With, system::Local},
    prelude::{Query, Res, ResMut},
    render::renderer::{RenderDevice, RenderQueue},
    time::Time,
};

use crate::tilemap::despawn::{DespawnedTile, DespawnedTilemap};

use super::{
    binding::TilemapBindGroups,
    buffer::{
        PerTilemapBuffersStorage, StandardMaterialUniformBuffer, TilemapStorageBuffers,
        TilemapUniformBuffer, UniformBuffer,
    },
    chunk::{TilemapRenderChunk, UnloadRenderChunk},
    extract::{ExtractedTile, TilemapInstance},
    material::{
        ExtractedStandardTilemapMaterials, PrepareNextFrameStdTilemapMaterials,
        StandardTilemapMaterialInstances,
    },
    pipeline::EntiTilesPipeline,
    resources::TilemapInstances,
    texture::TilemapTexturesStorage,
    RenderChunkStorage,
};

pub fn prepare_tilemaps(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    extracted_tilemaps: Query<Entity, With<TilemapInstance>>,
    mut render_chunks: ResMut<RenderChunkStorage>,
    mut uniform_buffers: ResMut<TilemapUniformBuffer>,
    mut storage_buffers: ResMut<TilemapStorageBuffers>,
    mut textures_storage: ResMut<TilemapTexturesStorage>,
    entitiles_pipeline: Res<EntiTilesPipeline>,
    mut bind_groups: ResMut<TilemapBindGroups>,
    tilemap_instances: Res<TilemapInstances>,
    materials: Res<StandardTilemapMaterialInstances>,
    std_material_uniform_buffer: Res<StandardMaterialUniformBuffer>,
    time: Res<Time>,
) {
    uniform_buffers.clear();
    storage_buffers.clear();

    extracted_tilemaps
        .iter()
        .filter_map(|tilemap| tilemap_instances.0.get(&tilemap))
        .for_each(|tilemap| {
            uniform_buffers.insert(&(tilemap, time.elapsed_seconds()), tilemap.id);
            render_chunks.prepare_chunks(tilemap, &render_device);

            if let Some(material) = materials.get(&tilemap.material) {
                if material.texture.is_none() {
                    return;
                }

                storage_buffers
                    .get_or_insert_buffer(tilemap.id)
                    .extend(&tilemap.animations.as_ref().unwrap().0);

                if !textures_storage.contains(&tilemap.material) {
                    textures_storage.insert(tilemap.material.clone());
                }
            }
        });

    #[cfg(not(feature = "atlas"))]
    textures_storage.prepare_textures(&render_device, &materials);
    uniform_buffers.write(&render_device, &render_queue);
    storage_buffers.write(&render_device, &render_queue);

    bind_groups.bind_uniform_buffers(
        &render_device,
        &mut uniform_buffers,
        &entitiles_pipeline,
        &std_material_uniform_buffer,
    );
    bind_groups.bind_storage_buffers(&render_device, &mut storage_buffers, &entitiles_pipeline);
}

pub fn prepare_std_materials(
    mut prepare_next_frame: Local<PrepareNextFrameStdTilemapMaterials>,
    mut extracted_materials: ResMut<ExtractedStandardTilemapMaterials>,
    mut material_instances: ResMut<StandardTilemapMaterialInstances>,
    mut std_material_uniform_buffer: ResMut<StandardMaterialUniformBuffer>,
    mut bind_groups: ResMut<TilemapBindGroups>,
    textures_storage: Res<TilemapTexturesStorage>,
    entitiles_pipeline: Res<EntiTilesPipeline>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    let is_dirty = !extracted_materials.removed.is_empty()
        || !extracted_materials.extracted.is_empty()
        || !prepare_next_frame.assets.is_empty();

    if is_dirty {
        std_material_uniform_buffer.clear();
    }

    for removed in std::mem::take(&mut extracted_materials.removed) {
        material_instances.remove(&removed);
    }

    for (asset_id, material) in std::mem::take(&mut extracted_materials.extracted) {
        material_instances.insert(asset_id, material.clone());
        std_material_uniform_buffer.insert(&material, asset_id);

        if !bind_groups.prepare_materials(
            &asset_id,
            &render_device,
            &textures_storage,
            &entitiles_pipeline,
        ) {
            prepare_next_frame.assets.push((asset_id, material));
            continue;
        }
    }

    for (asset_id, material) in std::mem::take(&mut prepare_next_frame.assets) {
        if !bind_groups.prepare_materials(
            &asset_id,
            &render_device,
            &textures_storage,
            &entitiles_pipeline,
        ) {
            prepare_next_frame.assets.push((asset_id, material));
            continue;
        }
    }

    if is_dirty {
        std_material_uniform_buffer.write(&render_device, &render_queue);
    }
}

pub fn prepare_tiles(
    extracted_tiles: Query<&mut ExtractedTile>,
    mut render_chunks: ResMut<RenderChunkStorage>,
    tilemap_instances: Res<TilemapInstances>,
    materials: Res<StandardTilemapMaterialInstances>,
) {
    extracted_tiles.iter().for_each(|tile| {
        let Some(tilemap) = tilemap_instances.0.get(&tile.tilemap_id) else {
            return;
        };

        let chunks = render_chunks.value.entry(tile.tilemap_id).or_default();

        let chunk = chunks.entry(tile.chunk_index).or_insert_with(|| {
            TilemapRenderChunk::from_index(tile.chunk_index, tilemap, &materials)
        });

        chunk.set_tile(tile.in_chunk_index, Some(tile));
    });
}

pub fn prepare_unloaded_chunks(
    mut render_chunks: ResMut<RenderChunkStorage>,
    extracted_tilemaps: Query<(Entity, &UnloadRenderChunk)>,
) {
    extracted_tilemaps.iter().for_each(|(entity, unloaded)| {
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
    tilemaps_query.iter().for_each(|map| {
        render_chunks.remove_tilemap(map.0);
        storage_buffers.remove(map.0);
        tilemap_instaces.0.remove(&map.0);
    });
}

pub fn prepare_despawned_tiles(
    mut render_chunks: ResMut<RenderChunkStorage>,
    tiles_query: Query<&DespawnedTile>,
) {
    tiles_query.iter().for_each(|tile| {
        if let Some(chunk) = render_chunks
            .get_chunks_mut(tile.tilemap)
            .and_then(|chunks| chunks.get_mut(&tile.chunk_index))
        {
            chunk.set_tile(tile.in_chunk_index, None);
        }
    });
}
