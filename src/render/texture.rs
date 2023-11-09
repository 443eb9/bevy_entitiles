use bevy::{
    prelude::{Assets, Commands, Handle, Image, Query, Res, ResMut, Resource, UVec2, Vec2, With},
    render::{
        render_asset::RenderAssets,
        render_resource::{
            AddressMode, Extent3d, FilterMode, ImageCopyTexture, Origin3d, SamplerDescriptor,
            TextureAspect, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
            TextureViewDescriptor, TextureViewDimension,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::GpuImage,
    },
    utils::{HashMap, HashSet},
};

use crate::tilemap::{TileTexture, Tilemap, WaitForTextureUsageChange};

#[derive(Resource, Default)]
pub struct TilemapTextureArrayStorage {
    textures: HashMap<Handle<Image>, GpuImage>,
    descs: HashMap<Handle<Image>, TilemapTextureDescriptor>,
    textures_to_prepare: HashSet<Handle<Image>>,
    textures_to_queue: HashSet<Handle<Image>>,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct TilemapTextureDescriptor {
    pub tile_count: UVec2,
    pub tile_size: UVec2,
    pub filter_mode: FilterMode,
}

impl TilemapTextureArrayStorage {
    /// Register a new image to be translated to texture array.
    pub fn insert_texture(&mut self, texture: TileTexture) {
        if !self.descs.contains_key(&texture.clone_weak()) {
            self.textures_to_prepare.insert(texture.clone_weak());
            self.descs.insert(texture.clone_weak(), *texture.get_desc());
        }
    }

    /// Try to get the processed texture array.
    pub fn get_texture_array(&self, image: &Handle<Image>) -> Option<&GpuImage> {
        self.textures.get(image)
    }

    /// Prepare the texture, creating the texture array and translate images in `queue_texture` function.
    pub fn prepare_textures(&mut self, render_device: &Res<RenderDevice>) {
        if self.textures_to_prepare.is_empty() {
            return;
        }

        let to_prepare = self.textures_to_prepare.drain().collect::<Vec<_>>();

        for image_handle in to_prepare.iter() {
            if image_handle.id() == Handle::<Image>::default().id() {
                continue;
            }

            let desc = self.descs.get(image_handle).unwrap();

            let texture = render_device.create_texture(&TextureDescriptor {
                label: Some("tilemap_texture_array"),
                size: Extent3d {
                    width: desc.tile_size.x,
                    height: desc.tile_size.y,
                    depth_or_array_layers: desc.tile_count.x * desc.tile_count.y,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8UnormSrgb,
                usage: TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });

            let sampler = render_device.create_sampler(&SamplerDescriptor {
                label: Some("tilemap_texture_array_sampler"),
                address_mode_u: AddressMode::ClampToEdge,
                address_mode_v: AddressMode::ClampToEdge,
                address_mode_w: AddressMode::ClampToEdge,
                mag_filter: desc.filter_mode,
                min_filter: desc.filter_mode,
                mipmap_filter: desc.filter_mode,
                lod_min_clamp: 0.,
                lod_max_clamp: f32::MAX,
                compare: None,
                anisotropy_clamp: 1,
                border_color: None,
            });

            let texture_view = texture.create_view(&TextureViewDescriptor {
                label: Some("tilemap_texture_array_view"),
                format: Some(TextureFormat::Rgba8UnormSrgb),
                dimension: Some(TextureViewDimension::D2Array),
                aspect: TextureAspect::All,
                base_mip_level: 0,
                base_array_layer: 0,
                mip_level_count: None,
                array_layer_count: Some(desc.tile_count.x * desc.tile_count.y),
            });

            let gpu_image = GpuImage {
                texture_format: texture.format(),
                mip_level_count: texture.mip_level_count(),
                texture,
                texture_view,
                sampler,
                size: Vec2::new(desc.tile_size.x as f32, desc.tile_size.y as f32),
            };

            self.textures.insert(image_handle.clone_weak(), gpu_image);
            self.textures_to_queue.insert(image_handle.clone_weak());
        }
    }

    /// Translate images to texture array.
    pub fn queue_textures(
        &mut self,
        render_device: &Res<RenderDevice>,
        render_queue: &Res<RenderQueue>,
        render_images: &Res<RenderAssets<Image>>,
    ) {
        if self.textures_to_queue.is_empty() {
            return;
        }

        let to_queue = self.textures_to_queue.drain().collect::<Vec<_>>();

        for image in to_queue.iter() {
            let Some(raw_gpu_image) = render_images.get(image) else {
                self.textures_to_prepare.insert(image.clone_weak());
                continue;
            };

            let desc = self.descs.get(image).unwrap();
            let array_gpu_image = self.textures.get(image).unwrap();
            let mut command_encoder = render_device.create_command_encoder(&Default::default());

            for index_y in 0..desc.tile_count.y {
                for index_x in 0..desc.tile_count.x {
                    command_encoder.copy_texture_to_texture(
                        ImageCopyTexture {
                            texture: &raw_gpu_image.texture,
                            mip_level: 0,
                            origin: Origin3d {
                                x: index_x * desc.tile_size.x,
                                y: index_y * desc.tile_size.y,
                                z: 0,
                            },
                            aspect: TextureAspect::All,
                        },
                        ImageCopyTexture {
                            texture: &array_gpu_image.texture,
                            mip_level: 0,
                            origin: Origin3d {
                                x: 0,
                                y: 0,
                                z: index_x + index_y * desc.tile_count.x,
                            },
                            aspect: TextureAspect::All,
                        },
                        Extent3d {
                            width: desc.tile_size.x,
                            height: desc.tile_size.y,
                            depth_or_array_layers: 1,
                        },
                    );
                }
            }

            render_queue.submit(vec![command_encoder.finish()]);
        }
    }
}

pub fn set_texture_usage(
    mut commands: Commands,
    mut tilemaps_query: Query<&mut Tilemap, With<WaitForTextureUsageChange>>,
    mut image_assets: ResMut<Assets<Image>>,
) {
    for mut tilemap in tilemaps_query.iter_mut() {
        tilemap.set_usage(&mut commands, &mut image_assets);
    }
}
