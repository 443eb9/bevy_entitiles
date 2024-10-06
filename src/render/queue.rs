use bevy::{
    core_pipeline::core_2d::Transparent2d,
    ecs::query::With,
    math::FloatOrd,
    prelude::{Commands, Entity, Msaa, Query, Res, ResMut},
    render::{
        camera::ExtractedCamera,
        render_asset::RenderAssets,
        render_phase::{DrawFunctions, PhaseItemExtraIndex, ViewSortedRenderPhases},
        render_resource::{BindGroupEntry, PipelineCache, SpecializedRenderPipelines},
        renderer::{RenderDevice, RenderQueue},
        texture::GpuImage,
        view::ViewUniforms,
    },
};

use crate::{
    render::{
        binding::{TilemapBindGroups, TilemapViewBindGroup},
        draw::{DrawTilemapNonTextured, DrawTilemapTextured},
        extract::TilemapInstance,
        material::TilemapMaterial,
        pipeline::{EntiTilesPipeline, EntiTilesPipelineKey},
        resources::TilemapInstances,
        texture::TilemapTexturesStorage,
    },
    tilemap::map::TilemapTextures,
};

pub fn queue_textures(
    mut textures_storage: ResMut<TilemapTexturesStorage>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    textures_assets: Res<RenderAssets<TilemapTextures>>,
    #[cfg(not(feature = "atlas"))] render_images: Res<RenderAssets<GpuImage>>,
    #[cfg(feature = "atlas")] mut render_images: ResMut<RenderAssets<GpuImage>>,
) {
    // #[cfg(not(feature = "atlas"))]
    // textures_storage.queue_textures(
    //     &render_device,
    //     &render_queue,
    //     &render_images,
    //     &textures_assets,
    // );
    #[cfg(feature = "atlas")]
    textures_storage.queue_textures(
        &render_device,
        &render_queue,
        &mut render_images,
        &textures_assets,
    );
}

pub fn queue_tilemaps<M: TilemapMaterial>(
    mut commands: Commands,
    mut views_query: Query<Entity, With<ExtractedCamera>>,
    tilemaps_query: Query<Entity, With<TilemapInstance>>,
    pipeline_cache: Res<PipelineCache>,
    draw_functions: Res<DrawFunctions<Transparent2d>>,
    mut sp_entitiles_pipeline: ResMut<SpecializedRenderPipelines<EntiTilesPipeline<M>>>,
    entitiles_pipeline: Res<EntiTilesPipeline<M>>,
    view_uniforms: Res<ViewUniforms>,
    render_device: Res<RenderDevice>,
    msaa: Res<Msaa>,
    tilemap_instances: Res<TilemapInstances<M>>,
    mut transparent_phase: ResMut<ViewSortedRenderPhases<Transparent2d>>,
) {
    let Some(view_binding) = view_uniforms.uniforms.binding() else {
        return;
    };

    for view_entity in views_query.iter_mut() {
        let Some(transparent_phase) = transparent_phase.get_mut(&view_entity) else {
            continue;
        };

        commands.entity(view_entity).insert(TilemapViewBindGroup {
            value: render_device.create_bind_group(
                "tilemap_view_bind_group",
                &entitiles_pipeline.view_layout,
                &[BindGroupEntry {
                    binding: 0,
                    resource: view_binding.clone(),
                }],
            ),
        });

        let mut tilemaps = tilemaps_query
            .iter()
            .filter_map(|t| tilemap_instances.0.get(&t))
            .collect::<Vec<_>>();
        radsort::sort_by_key(&mut tilemaps, |m| m.transform.z_index);

        for tilemap in tilemaps.iter() {
            let pipeline = sp_entitiles_pipeline.specialize(
                &pipeline_cache,
                &entitiles_pipeline,
                EntiTilesPipelineKey {
                    msaa: msaa.samples(),
                    map_type: tilemap.ty,
                    is_pure_color: tilemap.texture.is_none(),
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
                entity: tilemap.id,
                pipeline,
                draw_function,
                batch_range: 0..1,
                extra_index: PhaseItemExtraIndex::NONE,
            });
        }
    }
}
