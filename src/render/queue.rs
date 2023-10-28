use bevy::{
    prelude::{Res, Query},
    render::{
        render_resource::{BindGroupDescriptor, BindGroupEntry, SpecializedRenderPipeline},
        renderer::RenderDevice,
    },
};

use super::{BindGroups, EntiTilesPipeline, EntiTilesPipelineKey, UniformData};

pub fn queue(
    render_device: Res<RenderDevice>,
    uniform_data: Res<UniformData>,
    mut bind_groups: Query<&mut BindGroups>,
    pipeline: Res<EntiTilesPipeline>,
) {
    if let Some(tile_data_binding) = uniform_data.tile_data.binding() {
        bind_groups.get_single_mut().unwrap().tile_data = render_device.create_bind_group(&BindGroupDescriptor {
            label: Some("tilemap_bind_group"),
            layout: &pipeline.mesh_layout,
            entries: &[BindGroupEntry {
                binding: 1,
                resource: tile_data_binding,
            }],
        });
    }

    let specialize_pipeline = pipeline.specialize(EntiTilesPipelineKey {});
}
