use bevy::{
    ecs::{entity::Entity, query::With},
    prelude::{Commands, Query, Res, ResMut},
    render::{
        render_asset::RenderAssets,
        renderer::{RenderDevice, RenderQueue},
        texture::{FallbackImage, Image},
    },
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
    material::TilemapMaterial,
    pipeline::EntiTilesPipeline,
    resources::{ExtractedTilemapMaterials, TilemapInstances},
    texture::TilemapTexturesStorage,
    RenderChunkStorage,
};

pub fn prepare_tilemaps<M: TilemapMaterial>(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    extracted_tilemaps: Query<Entity, With<TilemapInstance>>,
    mut render_chunks: ResMut<RenderChunkStorage<M>>,
    mut uniform_buffers: ResMut<TilemapUniformBuffer<M>>,
    mut storage_buffers: ResMut<TilemapStorageBuffers>,
    mut textures_storage: ResMut<TilemapTexturesStorage>,
    entitiles_pipeline: Res<EntiTilesPipeline<M>>,
    mut bind_groups: ResMut<TilemapBindGroups<M>>,
    time: Res<Time>,
    tilemap_instances: Res<TilemapInstances<M>>,
    images: Res<RenderAssets<Image>>,
    fallback_image: Res<FallbackImage>,
    extracted_materials: Res<ExtractedTilemapMaterials<M>>,
) {
    uniform_buffers.clear();
    storage_buffers.clear();

    extracted_tilemaps
        .iter()
        .filter_map(|tilemap| tilemap_instances.0.get(&tilemap))
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
    bind_groups.prepare_material_bind_groups(
        &entitiles_pipeline.material_layout,
        &render_device,
        &images,
        &fallback_image,
        &extracted_materials,
    );
}

pub fn prepare_tiles<M: TilemapMaterial>(
    extracted_tiles: Query<&mut ExtractedTile>,
    mut render_chunks: ResMut<RenderChunkStorage<M>>,
    tilemap_instances: Res<TilemapInstances<M>>,
) {
    extracted_tiles.iter().for_each(|tile| {
        let Some(tilemap) = tilemap_instances.0.get(&tile.tilemap_id) else {
            return;
        };

        let chunks = render_chunks.value.entry(tile.tilemap_id).or_default();

        let chunk = chunks
            .entry(tile.chunk_index)
            .or_insert_with(|| TilemapRenderChunk::from_index(tile.chunk_index, tilemap));

        chunk.set_tile(tile.in_chunk_index, Some(tile));
    });
}

pub fn prepare_unloaded_chunks<M: TilemapMaterial>(
    mut render_chunks: ResMut<RenderChunkStorage<M>>,
    extracted_tilemaps: Query<(Entity, &UnloadRenderChunk)>,
) {
    extracted_tilemaps.iter().for_each(|(entity, unloaded)| {
        unloaded.0.iter().for_each(|c| {
            render_chunks.remove_chunk(entity, *c);
        });
    });
}

pub fn prepare_despawned_tilemaps<M: TilemapMaterial>(
    mut render_chunks: ResMut<RenderChunkStorage<M>>,
    mut storage_buffers: ResMut<TilemapStorageBuffers>,
    mut tilemap_instaces: ResMut<TilemapInstances<M>>,
    tilemaps_query: Query<&DespawnedTilemap>,
) {
    tilemaps_query.iter().for_each(|map| {
        render_chunks.remove_tilemap(map.0);
        storage_buffers.remove(map.0);
        tilemap_instaces.0.remove(&map.0);
    });
}

pub fn prepare_despawned_tiles<M: TilemapMaterial>(
    mut render_chunks: ResMut<RenderChunkStorage<M>>,
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
