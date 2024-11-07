use bevy::{
    core_pipeline::core_2d::Transparent2d,
    ecs::query::With,
    math::FloatOrd,
    prelude::{Entity, Msaa, Query, Res, ResMut},
    render::{
        camera::ExtractedCamera,
        render_phase::{DrawFunctions, PhaseItemExtraIndex, ViewSortedRenderPhases},
        render_resource::{PipelineCache, SpecializedRenderPipelines},
    },
};

use crate::render::{
    draw::{DrawTilemapNonTextured, DrawTilemapTextured},
    extract::{TilemapInstances, TilemapMaterialIds},
    material::TilemapMaterial,
    pipeline::{EntiTilesPipeline, EntiTilesPipelineKey},
};

pub fn queue_tilemaps<M: TilemapMaterial>(
    mut views_query: Query<Entity, With<ExtractedCamera>>,
    pipeline_cache: Res<PipelineCache>,
    draw_functions: Res<DrawFunctions<Transparent2d>>,
    mut sp_entitiles_pipeline: ResMut<SpecializedRenderPipelines<EntiTilesPipeline<M>>>,
    entitiles_pipeline: Res<EntiTilesPipeline<M>>,
    msaa: Res<Msaa>,
    tilemap_instances: Res<TilemapInstances>,
    mut transparent_phase: ResMut<ViewSortedRenderPhases<Transparent2d>>,
    material_ids: Res<TilemapMaterialIds<M>>,
    #[cfg(target_arch = "wasm32")] render_device: Res<bevy::render::renderer::RenderDevice>,
) {
    for view_entity in views_query.iter_mut() {
        let Some(transparent_phase) = transparent_phase.get_mut(&view_entity) else {
            continue;
        };

        // TODO optimize this
        let mut tilemaps = tilemap_instances
            .iter()
            .filter(|(t, _)| material_ids.contains_key(*t))
            .collect::<Vec<_>>();
        radsort::sort_by_key(&mut tilemaps, |(_, m)| m.transform.z_index);

        for (entity, tilemap) in tilemaps {
            let pipeline =
                sp_entitiles_pipeline.specialize(
                    &pipeline_cache,
                    &entitiles_pipeline,
                    EntiTilesPipelineKey {
                        msaa: msaa.samples(),
                        map_type: tilemap.ty,
                        is_pure_color: tilemap.texture.is_none(),
                        #[cfg(target_arch = "wasm32")]
                        anim_seq_len: bevy::render::render_resource::GpuArrayBuffer::<
                            bevy::math::IVec4,
                        >::batch_size(&render_device)
                        .unwrap(),
                        #[cfg(target_arch = "wasm32")]
                        tex_desc_len: bevy::render::render_resource::GpuArrayBuffer::<
                            crate::render::buffer::GpuTilemapTextureDescriptor,
                        >::batch_size(&render_device)
                        .unwrap(),
                    },
                );

            let draw_function = {
                if tilemap.texture.is_none() {
                    draw_functions
                        .read()
                        .get_id::<DrawTilemapNonTextured<M>>()
                        .unwrap()
                } else {
                    draw_functions
                        .read()
                        .get_id::<DrawTilemapTextured<M>>()
                        .unwrap()
                }
            };

            transparent_phase.add(Transparent2d {
                sort_key: FloatOrd(tilemap.transform.z_index as f32),
                entity: *entity,
                pipeline,
                draw_function,
                batch_range: 0..1,
                extra_index: PhaseItemExtraIndex::NONE,
            });
        }
    }
}
