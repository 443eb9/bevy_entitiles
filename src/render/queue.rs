use bevy::{
    core_pipeline::core_2d::Transparent2d,
    prelude::{Entity, Query, Res, ResMut},
    render::{
        render_phase::{DrawFunctions, RenderPhase},
        render_resource::{
            PipelineCache, SpecializedRenderPipelines,
        },
    },
    utils::FloatOrd,
};

use crate::tilemap::Tilemap;

use super::{
    draw::DrawTilemap,
    pipeline::{EntiTilesPipeline, EntiTilesPipelineKey},
};

pub fn queue(
    mut views_query: Query<&mut RenderPhase<Transparent2d>>,
    tilemaps_query: Query<(Entity, &Tilemap)>,
    pipeline_cache: Res<PipelineCache>,
    draw_functions: Res<DrawFunctions<Transparent2d>>,
    mut tilemap_pipelines: ResMut<SpecializedRenderPipelines<EntiTilesPipeline>>,
    entitile_pipeline: Res<EntiTilesPipeline>,
) {
    for mut transparent_phase in views_query.iter_mut() {
        for (tilemap_entity, tilemap_data) in tilemaps_query.iter() {
            let pipeline = tilemap_pipelines.specialize(
                &pipeline_cache,
                &entitile_pipeline,
                EntiTilesPipelineKey {
                    map_type: tilemap_data.tile_type.clone(),
                },
            );

            transparent_phase.add(Transparent2d {
                sort_key: FloatOrd(tilemap_data.z_order),
                entity: tilemap_entity,
                pipeline,
                draw_function: draw_functions.read().get_id::<DrawTilemap>().unwrap(),
                batch_range: None,
            });
        }
    }
}
