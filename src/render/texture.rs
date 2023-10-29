use bevy::{
    prelude::{Assets, Image, Query, Res, ResMut},
    render::{
        render_resource::{
            BindGroupDescriptor, BindGroupEntry, BindingResource, Extent3d, TextureAspect,
            TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
            TextureViewDescriptor, TextureViewDimension,
        },
        renderer::RenderDevice,
    }
};

use crate::tilemap::Tilemap;

use super::{BindGroups, EntiTilesPipeline};

pub fn prepare_textures(
    query: Query<&Tilemap>,
    mut bind_groups: ResMut<BindGroups>,
    render_device: Res<RenderDevice>,
    pipeline: Res<EntiTilesPipeline>,
    images: Res<Assets<Image>>,
) {
    for map in query.iter() {
        let texture_handle = map.texture.clone();

        if bind_groups.tile_textures.contains_key(&texture_handle) {
            continue;
        }

        let texture_desc = &images.get(&texture_handle).unwrap().texture_descriptor;

        let gpu_texture = render_device.create_texture(&TextureDescriptor {
            label: Some("tilemap_texture"),
            size: Extent3d {
                width: texture_desc.size.width,
                height: texture_desc.size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let tilemap_texture_bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: Some("tilemap_texture_bind_group"),
            layout: &pipeline.texture_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&gpu_texture.create_view(
                    &TextureViewDescriptor {
                        label: Some("tilemap_texture_view"),
                        format: None,
                        dimension: Some(TextureViewDimension::D2Array),
                        aspect: TextureAspect::All,
                        base_mip_level: 0,
                        mip_level_count: None,
                        base_array_layer: 0,
                        array_layer_count: None,
                    },
                )),
            }],
        });

        bind_groups
            .tile_textures
            .insert(texture_handle, tilemap_texture_bind_group);
    }
}
