use bevy::{
    asset::{Assets, Handle},
    ecs::{
        entity::Entity,
        query::With,
        system::{Commands, Query, ResMut, Resource},
    },
    prelude::Image,
    render::{
        render_asset::RenderAssets,
        render_resource::{AddressMode, SamplerDescriptor, TextureUsages},
        renderer::RenderDevice,
        texture::GpuImage,
    },
    utils::HashMap,
};

#[cfg(not(feature = "atlas"))]
use bevy::{
    math::Vec2,
    render::{
        render_resource::{
            Extent3d, ImageCopyTexture, Origin3d, TextureAspect, TextureDescriptor,
            TextureDimension, TextureFormat, TextureViewDescriptor, TextureViewDimension,
        },
        renderer::RenderQueue,
    },
};

use crate::tilemap::map::{TilemapTexture, TilemapTextureDescriptor, WaitForTextureUsageChange};

#[derive(Resource, Default)]
pub struct TilemapTexturesStorage {
    textures: HashMap<Handle<Image>, GpuImage>,
    prepare_queue: HashMap<Handle<Image>, TilemapTextureDescriptor>,
    queue_queue: HashMap<Handle<Image>, TilemapTextureDescriptor>,
}

impl TilemapTexturesStorage {
    pub fn insert(&mut self, handle: Handle<Image>, desc: &TilemapTextureDescriptor) {
        #[cfg(not(feature = "atlas"))]
        self.prepare_queue.insert(handle, desc.clone());
        #[cfg(feature = "atlas")]
        self.queue_queue.insert(handle, desc.clone());
    }

    /// Try to get the processed texture array.
    pub fn get_texture(&self, image: &Handle<Image>) -> Option<&GpuImage> {
        self.textures.get(image)
    }

    /// Prepare the texture, creating the texture array and translate images in `queue_texture` function.
    #[cfg(not(feature = "atlas"))]
    pub fn prepare_textures(&mut self, render_device: &RenderDevice) {
        if self.prepare_queue.is_empty() {
            return;
        }

        let to_prepare = self.prepare_queue.drain().collect::<Vec<_>>();

        for (image_handle, desc) in to_prepare.iter() {
            if image_handle.id() == Handle::<Image>::default().id() {
                continue;
            }

            let tile_count = desc.size / desc.tile_size;

            let texture = render_device.create_texture(&TextureDescriptor {
                label: Some("tilemap_texture_array"),
                size: Extent3d {
                    width: desc.tile_size.x,
                    height: desc.tile_size.y,
                    depth_or_array_layers: tile_count.x * tile_count.y,
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
                array_layer_count: Some(tile_count.x * tile_count.y),
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
            self.queue_queue
                .insert(image_handle.clone_weak(), desc.clone());
        }
    }

    /// Translate images to texture array.
    #[cfg(not(feature = "atlas"))]
    pub fn queue_textures(
        &mut self,
        render_device: &RenderDevice,
        render_queue: &RenderQueue,
        render_images: &RenderAssets<Image>,
    ) {
        if self.queue_queue.is_empty() {
            return;
        }

        let to_queue = self.queue_queue.drain().collect::<Vec<_>>();

        for (image_handle, desc) in to_queue.iter() {
            let Some(raw_gpu_image) = render_images.get(image_handle) else {
                self.queue_queue
                    .insert(image_handle.clone_weak(), desc.clone());
                continue;
            };

            if !raw_gpu_image
                .texture
                .usage()
                .contains(TextureUsages::COPY_SRC)
            {
                self.queue_queue
                    .insert(image_handle.clone_weak(), desc.clone());
                continue;
            }

            let tile_count = desc.size / desc.tile_size;
            let array_gpu_image = self.textures.get(image_handle).unwrap();
            let mut command_encoder = render_device.create_command_encoder(&Default::default());

            for index_y in 0..tile_count.y {
                for index_x in 0..tile_count.x {
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
                                z: index_x + index_y * tile_count.x,
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

    #[cfg(feature = "atlas")]
    pub fn queue_textures(
        &mut self,
        render_device: &RenderDevice,
        render_images: &mut RenderAssets<Image>,
    ) {
        if self.queue_queue.is_empty() {
            return;
        }

        let to_queue = self.queue_queue.drain().collect::<Vec<_>>();

        for (image_handle, desc) in to_queue.into_iter() {
            let Some(texture) = render_images.get_mut(&image_handle) else {
                self.queue_queue.insert(image_handle, desc);
                continue;
            };

            let sampler = render_device.create_sampler(&SamplerDescriptor {
                label: Some("tilemap_texture_atlas_sampler"),
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

            texture.sampler = sampler;
            self.textures.insert(image_handle, texture.clone());
        }
    }

    pub fn contains(&self, handle: &Handle<Image>) -> bool {
        self.textures.contains_key(handle)
            || self.queue_queue.contains_key(handle)
            || self.prepare_queue.contains_key(handle)
    }
}

pub fn set_texture_usage(
    mut commands: Commands,
    tilemaps_query: Query<(Entity, &TilemapTexture), With<WaitForTextureUsageChange>>,
    mut image_assets: ResMut<Assets<Image>>,
) {
    // Bevy doesn't set the `COPY_SRC` usage for images by default, so we need to do it manually.
    tilemaps_query.for_each(|(entity, tex)| {
        let Some(image) = image_assets.get(&tex.clone_weak()) else {
            return;
        };

        if !image
            .texture_descriptor
            .usage
            .contains(TextureUsages::COPY_SRC)
        {
            image_assets
                .get_mut(&tex.clone_weak())
                .unwrap()
                .texture_descriptor
                .usage
                .set(TextureUsages::COPY_SRC, true);
        }

        commands
            .entity(entity)
            .remove::<WaitForTextureUsageChange>();
    });
}
