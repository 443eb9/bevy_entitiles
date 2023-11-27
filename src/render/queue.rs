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
    extract::{ExtractedHeightTilemap, ExtractedTilemap},
    pipeline::{EntiTilesPipeline, EntiTilesPipelineKey},
    texture::TilemapTexturesStorage,
    uniform::TilemapUniformsStorage,
    TilemapBindGroups,
};

#[derive(Component)]
pub struct TileViewBindGroup {
    pub value: BindGroup,
}

pub fn queue(
    mut commands: Commands,
    mut views_query: Query<(Entity, &mut RenderPhase<Transparent2d>)>,
    tilemaps_query: Query<(Entity, &ExtractedTilemap, Option<&ExtractedHeightTilemap>)>,
    pipeline_cache: Res<PipelineCache>,
    draw_functions: Res<DrawFunctions<Transparent2d>>,
    mut sp_entitiles_pipeline: ResMut<SpecializedRenderPipelines<EntiTilesPipeline>>,
    entitile_pipeline: Res<EntiTilesPipeline>,
    view_uniforms: Res<ViewUniforms>,
    render_device: Res<RenderDevice>,
    mut bind_groups: ResMut<TilemapBindGroups>,
    textures_storage: Res<TilemapTexturesStorage>,
    msaa: Res<Msaa>,
    tilemap_uniform_strorage: Res<TilemapUniformsStorage>,
    #[cfg(feature = "post_processing")] mut post_processing_bind_groups: ResMut<
        crate::post_processing::PostProcessingBindGroups,
    >,
    #[cfg(feature = "post_processing")] post_processing_textures: Res<
        crate::post_processing::PostProcessingTextures,
    >,
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
        tilemaps.sort_by(|lhs, rhs| rhs.1.z_order.cmp(&lhs.1.z_order));

        let mut is_pure_color = true;
        let mut is_height_tilemap = false;

        for (entity, tilemap, height_tilemap) in tilemaps_query.iter() {
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
                let Some(texture) = textures_storage.get(tile_texture.handle()) else {
                    continue;
                };

                if !bind_groups
                    .color_textures
                    .contains_key(tile_texture.handle())
                {
                    let bind_group = render_device.create_bind_group(
                        Some("color_texture_bind_group"),
                        &entitile_pipeline.color_texture_layout,
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
                        .color_textures
                        .insert(tile_texture.clone_weak(), bind_group);
                }

                is_pure_color = false;
            }

            #[cfg(feature = "post_processing")]
            if let Some(height_tilemap) = height_tilemap {
                let height_texture = &height_tilemap.height_texture;
                if let Some(texture) = textures_storage.get(height_texture.handle()) {
                    if !post_processing_bind_groups
                        .height_texture_bind_groups
                        .contains_key(height_texture.handle())
                    {
                        let bind_group = render_device.create_bind_group(
                            Some("height_texture_bind_group"),
                            &entitile_pipeline.color_texture_layout,
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

                        post_processing_bind_groups
                            .height_texture_bind_groups
                            .insert(height_texture.clone_weak(), bind_group);
                    }
                }

                if post_processing_bind_groups
                    .screen_height_texture_bind_group
                    .is_none()
                {
                    post_processing_bind_groups.screen_height_texture_bind_group = Some(
                        render_device.create_bind_group(
                            Some("screen_height_texture_bind_group"),
                            &entitile_pipeline.screen_height_texture_layout,
                            &[BindGroupEntry {
                                binding: 0,
                                resource: BindingResource::TextureView(
                                    &post_processing_textures
                                        .screen_height_gpu_image
                                        .as_ref()
                                        .unwrap()
                                        .texture_view,
                                ),
                            }],
                        ),
                    );
                }

                is_height_tilemap = true;
            }

            let pipeline = sp_entitiles_pipeline.specialize(
                &pipeline_cache,
                &entitile_pipeline,
                EntiTilesPipelineKey {
                    msaa: msaa.samples(),
                    map_type: tilemap.tile_type,
                    flip: tilemap.flip,
                    is_pure_color,
                    is_height_tilemap,
                },
            );

            let mut draw_function = {
                if is_pure_color {
                    draw_functions
                        .read()
                        .get_id::<DrawTilemapPureColor>()
                        .unwrap()
                } else {
                    draw_functions.read().get_id::<DrawTilemap>().unwrap()
                }
            };

            #[cfg(feature = "post_processing")]
            if is_height_tilemap {
                draw_function = draw_functions
                    .read()
                    .get_id::<super::draw::DrawTilemapPostProcessing>()
                    .unwrap()
            }

            transparent_phase.add(Transparent2d {
                sort_key: FloatOrd(tilemap.z_order as f32),
                entity,
                pipeline,
                draw_function,
                batch_range: 0..1,
                dynamic_offset: NonMaxU32::new(0),
            });
        }
    }
}
