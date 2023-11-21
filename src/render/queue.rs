use bevy::{
    core_pipeline::core_2d::Transparent2d,
    prelude::{Commands, Component, Entity, Msaa, Query, Res, ResMut},
    render::{
        render_phase::{DrawFunctions, RenderPhase},
        render_resource::{
            BindGroup, BindGroupEntry, BindingResource, PipelineCache, SpecializedRenderPipelines,
        },
        renderer::RenderDevice,
        view::ViewUniforms,
    },
    utils::{nonmax::NonMaxU32, FloatOrd},
};

use super::{
    draw::{DrawTilemap, DrawTilemapPureColor},
    extract::ExtractedTilemap,
    pipeline::{EntiTilesPipeline, EntiTilesPipelineKey},
    uniform::TilemapUniformsStorage,
    TilemapBindGroups, texture::TilemapTexturesStorage,
};

#[derive(Component)]
pub struct TileViewBindGroup {
    pub value: BindGroup,
}

pub fn queue(
    mut commands: Commands,
    mut views_query: Query<(Entity, &mut RenderPhase<Transparent2d>)>,
    tilemaps_query: Query<(Entity, &ExtractedTilemap)>,
    pipeline_cache: Res<PipelineCache>,
    draw_functions: Res<DrawFunctions<Transparent2d>>,
    mut tilemap_pipelines: ResMut<SpecializedRenderPipelines<EntiTilesPipeline>>,
    entitile_pipeline: Res<EntiTilesPipeline>,
    view_uniforms: Res<ViewUniforms>,
    render_device: Res<RenderDevice>,
    mut bind_groups: ResMut<TilemapBindGroups>,
    textures_storage: Res<TilemapTexturesStorage>,
    msaa: Res<Msaa>,
    tilemap_uniform_strorage: Res<TilemapUniformsStorage>,
) {
    let Some(view_binding) = view_uniforms.uniforms.binding() else {
        return;
    };

    for (view_entity, mut transparent_phase) in views_query.iter_mut() {
        commands.entity(view_entity).insert(TileViewBindGroup {
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
        tilemaps.sort_by(|lhs, rhs| lhs.1.z_order.cmp(&rhs.1.z_order));

        for (entity, tilemap) in tilemaps.iter() {
            let pipeline = tilemap_pipelines.specialize(
                &pipeline_cache,
                &entitile_pipeline,
                EntiTilesPipelineKey {
                    msaa: msaa.samples(),
                    map_type: tilemap.tile_type,
                    flip: tilemap.flip,
                    is_pure_color: tilemap.texture.is_none(),
                },
            );

            if let Some(tilemap_uniform_binding) = tilemap_uniform_strorage.binding() {
                let tilemap_uniform_bind_group = render_device.create_bind_group(
                    Some("tilemap_data_bind_group"),
                    &entitile_pipeline.tilemap_uniform_layout,
                    &[BindGroupEntry {
                        binding: 0,
                        resource: tilemap_uniform_binding,
                    }],
                );

                bind_groups
                    .tilemap_uniform_bind_group
                    .insert(tilemap.id, tilemap_uniform_bind_group);
            }

            if let Some(tile_texture) = tilemap.texture.clone() {
                let Some(texture) = textures_storage.get(tile_texture.get_handle()) else {
                    continue;
                };

                let texture_bind_group = render_device.create_bind_group(
                    Some("tilemap_texture_bind_group"),
                    &entitile_pipeline.colored_texture_layout,
                    &[
                        BindGroupEntry {
                            binding: 0,
                            resource: BindingResource::TextureView(&texture.texture_view),
                        },
                        BindGroupEntry {
                            binding: 1,
                            resource: BindingResource::Sampler(&texture.sampler),
                        },
                    ],
                );

                bind_groups
                    .colored_textures
                    .insert(tile_texture.clone_weak(), texture_bind_group);

                transparent_phase.add(Transparent2d {
                    sort_key: FloatOrd(0.),
                    entity: *entity,
                    pipeline,
                    draw_function: draw_functions.read().get_id::<DrawTilemap>().unwrap(),
                    batch_range: 0..1,
                    dynamic_offset: NonMaxU32::new(0),
                });
            } else {
                transparent_phase.add(Transparent2d {
                    sort_key: FloatOrd(0.),
                    entity: *entity,
                    pipeline,
                    draw_function: draw_functions
                        .read()
                        .get_id::<DrawTilemapPureColor>()
                        .unwrap(),
                    batch_range: 0..1,
                    dynamic_offset: NonMaxU32::new(0),
                });
            }
        }
    }
}
