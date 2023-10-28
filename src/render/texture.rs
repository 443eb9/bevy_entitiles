use bevy::{
    prelude::{Assets, Commands, Image, Query, Res, ResMut},
    render::{
        render_resource::{
            BindGroupDescriptor, BindGroupEntry, BindingResource, Extent3d, TextureAspect,
            TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
            TextureViewDescriptor, TextureViewDimension,
        },
        renderer::RenderDevice,
    },
    utils::HashMap,
};

use crate::tilemap::TileMap;

use super::{BindGroups, EntiTilesPipeline};

pub fn prepare_textures(
    mut commands: Commands,
    query: Query<&TileMap>,
    mut bind_groups_query: Query<&mut BindGroups>,
    render_device: Res<RenderDevice>,
    pipeline: Res<EntiTilesPipeline>,
    images: Res<Assets<Image>>,
) {
    for map in query.iter() {
        let texture_handle = map.texture.clone();
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

        let tile_bind_group = render_device.create_bind_group(&BindGroupDescriptor {
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

        if let Ok(mut bind_groups) = bind_groups_query.get_single_mut() {
            bind_groups.tile_data = tile_bind_group;
        } else {
            commands.spawn_empty().insert(BindGroups {
                tile_data: tile_bind_group,
                tile_textures: HashMap::default(),
            });
        }
    }
}
