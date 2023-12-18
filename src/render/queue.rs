use bevy::{
    core_pipeline::core_2d::Transparent2d,
    prelude::{Commands, Entity, Msaa, Query, Res, ResMut},
    render::{
        render_asset::RenderAssets,
        render_phase::{DrawFunctions, RenderPhase},
        render_resource::{BindGroupEntry, PipelineCache, SpecializedRenderPipelines},
        renderer::{RenderDevice, RenderQueue},
        texture::Image,
        view::ViewUniforms,
    },
    utils::FloatOrd,
};

use super::{
    binding::{TilemapBindGroups, TilemapViewBindGroup},
    buffer::TilemapUniformBuffers,
    draw::{DrawTilemap, DrawTilemapPureColor},
    extract::ExtractedTilemap,
    pipeline::{EntiTilesPipeline, EntiTilesPipelineKey},
    texture::TilemapTexturesStorage,
};

pub fn queue(
    mut commands: Commands,
    mut views_query: Query<(Entity, &mut RenderPhase<Transparent2d>)>,
    tilemaps_query: Query<(Entity, &ExtractedTilemap)>,
    pipeline_cache: Res<PipelineCache>,
    draw_functions: Res<DrawFunctions<Transparent2d>>,
    mut sp_entitiles_pipeline: ResMut<SpecializedRenderPipelines<EntiTilesPipeline>>,
    entitile_pipeline: Res<EntiTilesPipeline>,
    view_uniforms: Res<ViewUniforms>,
    render_device: Res<RenderDevice>,
    mut bind_groups: ResMut<TilemapBindGroups>,
    mut textures_storage: ResMut<TilemapTexturesStorage>,
    msaa: Res<Msaa>,
    mut uniform_buffers: ResMut<TilemapUniformBuffers>,
    // mut storage_buffers: ResMut<TilemapStorageBuffers>,
    render_queue: Res<RenderQueue>,
    render_images: Res<RenderAssets<Image>>,
) {
    let Some(view_binding) = view_uniforms.uniforms.binding() else {
        return;
    };

    textures_storage.queue_textures(&render_device, &render_queue, &render_images);

    for (view_entity, mut transparent_phase) in views_query.iter_mut() {
        commands.entity(view_entity).insert(TilemapViewBindGroup {
            value: render_device.create_bind_group(
                "tilemap_view_bind_group",
                &entitile_pipeline.view_layout,
                &[BindGroupEntry {
                    binding: 0,
                    resource: view_binding.clone(),
                }],
            ),
        });

        let mut tilemaps = tilemaps_query.iter().collect::<Vec<_>>();
        radsort::sort_by_key(&mut tilemaps, |m| m.1.transform.z_index);

        for (entity, tilemap) in tilemaps.iter() {
            bind_groups.queue_uniform_buffers(
                &render_device,
                &mut uniform_buffers,
                &entitile_pipeline,
            );

            // bind_groups.queue_storage_buffers(
            //     tilemap,
            //     &render_device,
            //     &mut storage_buffers,
            //     &entitile_pipeline,
            // );

            let is_pure_color = bind_groups.queue_textures(
                &tilemap,
                &render_device,
                &textures_storage,
                &entitile_pipeline,
            );

            let pipeline = sp_entitiles_pipeline.specialize(
                &pipeline_cache,
                &entitile_pipeline,
                EntiTilesPipelineKey {
                    msaa: msaa.samples(),
                    map_type: tilemap.tile_type,
                    is_pure_color,
                },
            );

            let draw_function = {
                if is_pure_color {
                    draw_functions
                        .read()
                        .get_id::<DrawTilemapPureColor>()
                        .unwrap()
                } else {
                    draw_functions.read().get_id::<DrawTilemap>().unwrap()
                }
            };

            transparent_phase.add(Transparent2d {
                sort_key: FloatOrd(tilemap.transform.z_index as f32),
                entity: *entity,
                pipeline,
                draw_function,
                batch_range: 0..1,
                dynamic_offset: None,
            });
        }
    }
}
