use bevy::{
    core_pipeline::core_2d::Transparent2d,
    prelude::{Commands, Component, Entity, Image, Msaa, Query, Res, ResMut},
    render::{
        render_asset::RenderAssets,
        render_phase::{DrawFunctions, RenderPhase},
        render_resource::{
            BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource, PipelineCache,
            SpecializedRenderPipelines,
        },
        renderer::{RenderDevice, RenderQueue},
        view::ViewUniforms,
    },
    utils::FloatOrd,
};

use super::{
    draw::DrawTilemap,
    extract::ExtractedTilemap,
    pipeline::{EntiTilesPipeline, EntiTilesPipelineKey},
    texture::TilemapTextureArrayStorage,
    uniform::TilemapUniformsStorage,
    BindGroups,
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
    render_queue: Res<RenderQueue>,
    mut bind_groups: ResMut<BindGroups>,
    render_images: Res<RenderAssets<Image>>,
    mut tilemap_texture_array_storage: ResMut<TilemapTextureArrayStorage>,
    msaa: Res<Msaa>,
    tilemap_uniform_strorage: Res<TilemapUniformsStorage>,
) {
    let Some(view_binding) = view_uniforms.uniforms.binding() else {
        return;
    };

    tilemap_texture_array_storage.queue(&render_device, &render_queue, &render_images);

    for (view_entity, mut transparent_phase) in views_query.iter_mut() {
        commands.entity(view_entity).insert(TileViewBindGroup {
            value: render_device.create_bind_group(&BindGroupDescriptor {
                label: Some("tilemap_view_bind_group"),
                layout: &entitile_pipeline.view_layout,
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: view_binding.clone(),
                }],
            }),
        });

        for (entity, tilemap) in tilemaps_query.iter() {
            let pipeline = tilemap_pipelines.specialize(
                &pipeline_cache,
                &entitile_pipeline,
                EntiTilesPipelineKey {
                    msaa: msaa.samples(),
                    map_type: tilemap.tile_type.clone(),
                },
            );

            let Some(texture_array) =
                tilemap_texture_array_storage.try_get_texture_array(&tilemap.texture)
            else {
                continue;
            };

            let texture_bind_group = render_device.create_bind_group(&BindGroupDescriptor {
                label: Some("tilemap_texture_bind_group"),
                layout: &entitile_pipeline.texture_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&texture_array.texture_view),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Sampler(&texture_array.sampler),
                    },
                ],
            });

            bind_groups
                .tilemap_texture_arrays
                .insert(tilemap.texture.clone_weak(), texture_bind_group);

            if let Some(tilemap_uniform_binding) = tilemap_uniform_strorage.buffer.binding() {
                let tilemap_data_bind_group =
                    render_device.create_bind_group(&BindGroupDescriptor {
                        label: Some("tilemap_data_bind_group"),
                        layout: &entitile_pipeline.tilemap_data_layout,
                        entries: &[BindGroupEntry {
                            binding: 0,
                            resource: tilemap_uniform_binding,
                        }],
                    });

                bind_groups.tilemap_uniform_bind_group.insert(tilemap.id, tilemap_data_bind_group);
            }

            transparent_phase.add(Transparent2d {
                sort_key: FloatOrd(tilemap.z_order),
                entity,
                pipeline,
                draw_function: draw_functions.read().get_id::<DrawTilemap>().unwrap(),
                batch_range: None,
            });
        }
    }
}
