use bevy::{
    ecs::{entity::Entity, query::With},
    math::IVec2,
    prelude::{Commands, Query, Res, ResMut},
    render::{
        render_asset::RenderAssets,
        renderer::{RenderDevice, RenderQueue},
        texture::{FallbackImage, GpuImage},
    },
    time::Time,
};

use crate::{
    render::{
        binding::TilemapBindGroups,
        buffer::{
            PerTilemapBuffersStorage, TilemapAnimationBuffer, TilemapUniformBuffer, UniformBuffer,
        },
        chunk::{RenderChunkSort, UnloadRenderChunk},
        extract::{ExtractedTile, TilemapInstance},
        material::TilemapMaterial,
        pipeline::EntiTilesPipeline,
        resources::{ExtractedTilemapMaterials, TilemapInstances},
        texture::TilemapTexturesStorage,
        RenderChunkStorage,
    },
    tilemap::{
        despawn::{DespawnedTile, DespawnedTilemap},
        map::TilemapTextures,
    },
};

#[cfg(feature = "atlas")]
use super::buffer::TilemapTextureDescriptorBuffer;

pub fn prepare_tilemaps_a<M: TilemapMaterial>(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    extracted_tilemaps: Query<Entity, With<TilemapInstance>>,
    mut render_chunks: ResMut<RenderChunkStorage<M>>,
    mut uniform_buffers: ResMut<TilemapUniformBuffer<M>>,
    entitiles_pipeline: Res<EntiTilesPipeline<M>>,
    mut bind_groups: ResMut<TilemapBindGroups<M>>,
    time: Res<Time>,
    tilemap_instances: Res<TilemapInstances<M>>,
    images: Res<RenderAssets<GpuImage>>,
    fallback_image: Res<FallbackImage>,
    extracted_materials: Res<ExtractedTilemapMaterials<M>>,
) {
    uniform_buffers.clear();

    extracted_tilemaps
        .iter()
        .filter_map(|tilemap| tilemap_instances.0.get(&tilemap))
        .for_each(|tilemap| {
            commands
                .entity(tilemap.id)
                .insert(uniform_buffers.insert(&(tilemap, time.elapsed_seconds())));

            render_chunks.prepare_chunks(tilemap, &render_device);
        });

    uniform_buffers.write(&render_device, &render_queue);
    bind_groups.bind_uniform_buffers(&render_device, &mut uniform_buffers, &entitiles_pipeline);
    bind_groups.prepare_material_bind_groups(
        &entitiles_pipeline.material_layout,
        &render_device,
        &images,
        &fallback_image,
        &extracted_materials,
    );
}

pub fn prepare_tilemaps_b<M: TilemapMaterial>(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    extracted_tilemaps: Query<Entity, With<TilemapInstance>>,
    mut animation_buffers: ResMut<TilemapAnimationBuffer>,
    mut textures_storage: ResMut<TilemapTexturesStorage>,
    entitiles_pipeline: Res<EntiTilesPipeline<M>>,
    mut bind_groups: ResMut<TilemapBindGroups<M>>,
    tilemap_instances: Res<TilemapInstances<M>>,
    textures_assets: Res<RenderAssets<TilemapTextures>>,
    #[cfg(feature = "atlas")] mut texture_desc_buffers: ResMut<TilemapTextureDescriptorBuffer>,
) {
    animation_buffers.clear();
    #[cfg(feature = "atlas")]
    texture_desc_buffers.clear();

    extracted_tilemaps
        .iter()
        .filter_map(|tilemap| tilemap_instances.0.get(&tilemap))
        .filter(|tilemap| tilemap.texture.is_some())
        .for_each(|tilemap| {
            let textures_handle = tilemap.texture.as_ref().unwrap();
            animation_buffers
                .get_or_insert_buffer(tilemap.id)
                .extend(&tilemap.animations.as_ref().unwrap().0);

            let Some(_textures) = textures_assets.get(textures_handle) else {
                return;
            };

            #[cfg(feature = "atlas")]
            texture_desc_buffers
                .get_or_insert_buffer(tilemap.id)
                .extend(_textures.textures.iter().enumerate().map(|(i, t)| {
                    super::buffer::GpuTilemapTextureDescriptor {
                        tile_count: t.desc.size / t.desc.tile_size,
                        tile_uv_size: t.desc.tile_size.as_vec2() / t.desc.size.as_vec2(),
                        uv_scale: _textures.uv_scales[i],
                    }
                }));

            if !textures_storage.contains(textures_handle) {
                textures_storage.insert(textures_handle.clone());
            }
        });

    #[cfg(feature = "atlas")]
    texture_desc_buffers.write(&render_device, &render_queue);
    animation_buffers.write(&render_device, &render_queue);

    textures_storage.prepare_textures(&render_device, &textures_assets);
    bind_groups.bind_tilemap_storage_buffers(
        &render_device,
        &mut animation_buffers,
        &entitiles_pipeline,
        #[cfg(feature = "atlas")]
        &mut texture_desc_buffers,
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

        let chunks = render_chunks.get_or_insert_chunks(tilemap.id);
        chunks.try_add_chunk(tile.chunk_index, tilemap);
        chunks.set_tile(tile);
    });
}

pub fn prepare_unloaded_chunks<M: TilemapMaterial>(
    mut render_chunks: ResMut<RenderChunkStorage<M>>,
    extracted_tilemaps: Query<(Entity, &UnloadRenderChunk)>,
) {
    extracted_tilemaps.iter().for_each(|(entity, unloaded)| {
        unloaded.0.iter().for_each(|c| {
            render_chunks.get_or_insert_chunks(entity).remove_chunk(*c);
        });
    });
}

pub fn prepare_despawned_tilemaps<M: TilemapMaterial>(
    mut render_chunks: ResMut<RenderChunkStorage<M>>,
    mut storage_buffers: ResMut<TilemapAnimationBuffer>,
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
        render_chunks
            .get_or_insert_chunks(tile.tilemap)
            .remove_tile(tile.chunk_index, tile.in_chunk_index);
    });
}

pub fn sort_chunks<M: TilemapMaterial>(
    mut render_chunks: ResMut<RenderChunkStorage<M>>,
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
