use bevy::{
    core_pipeline::core_2d::Transparent2d,
    ecs::query::With,
    prelude::{Commands, Entity, Msaa, Query, Res, ResMut},
    render::{
        render_asset::RenderAssets,
        render_phase::{DrawFunctions, RenderPhase},
        render_resource::{BindGroupEntry, PipelineCache, SpecializedRenderPipelines},
        renderer::RenderDevice,
        texture::Image,
        view::ViewUniforms,
    },
    utils::FloatOrd,
};

use super::{
    binding::{TilemapBindGroups, TilemapViewBindGroup},
    draw::{DrawTilemap, DrawTilemapPureColor},
    extract::TilemapInstance,
    pipeline::{EntiTilesPipeline, EntiTilesPipelineKey},
    resources::TilemapInstances,
    texture::TilemapTexturesStorage,
};

#[cfg(not(feature = "atlas"))]
use bevy::render::renderer::RenderQueue;

pub fn queue(
    mut commands: Commands,
    mut views_query: Query<(Entity, &mut RenderPhase<Transparent2d>)>,
    tilemaps_query: Query<Entity, With<TilemapInstance>>,
    pipeline_cache: Res<PipelineCache>,
    draw_functions: Res<DrawFunctions<Transparent2d>>,
    mut sp_entitiles_pipeline: ResMut<SpecializedRenderPipelines<EntiTilesPipeline>>,
    entitiles_pipeline: Res<EntiTilesPipeline>,
    view_uniforms: Res<ViewUniforms>,
    render_device: Res<RenderDevice>,
    mut bind_groups: ResMut<TilemapBindGroups>,
    mut textures_storage: ResMut<TilemapTexturesStorage>,
    msaa: Res<Msaa>,
    tilemap_instances: Res<TilemapInstances>,
    #[cfg(not(feature = "atlas"))] render_queue: Res<RenderQueue>,
    #[cfg(not(feature = "atlas"))] render_images: Res<RenderAssets<Image>>,
    #[cfg(feature = "atlas")] mut render_images: ResMut<RenderAssets<Image>>,
) {
    let Some(view_binding) = view_uniforms.uniforms.binding() else {
        return;
    };

    #[cfg(not(feature = "atlas"))]
    textures_storage.queue_textures(&render_device, &render_queue, &render_images);
    #[cfg(feature = "atlas")]
    textures_storage.queue_textures(&render_device, &mut render_images);

    for (view_entity, mut transparent_phase) in views_query.iter_mut() {
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
            .filter_map(|t| tilemap_instances.get(t))
            .collect::<Vec<_>>();
        radsort::sort_by_key(&mut tilemaps, |m| m.transform.z_index);

        for tilemap in tilemaps.iter() {
            let is_pure_color = bind_groups.queue_textures(
                &tilemap,
                &render_device,
                &textures_storage,
                &entitiles_pipeline,
            );

            let pipeline = sp_entitiles_pipeline.specialize(
                &pipeline_cache,
                &entitiles_pipeline,
                EntiTilesPipelineKey {
                    msaa: msaa.samples(),
                    map_type: tilemap.ty,
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
                entity: tilemap.id,
                pipeline,
                draw_function,
                batch_range: 0..1,
                dynamic_offset: None,
            });
        }
    }
}
