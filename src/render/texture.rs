use bevy::{
    asset::{Assets, Handle},
    ecs::{
        entity::Entity,
        query::With,
        system::{Commands, Query, Res, ResMut, Resource},
    },
    prelude::Image,
    render::{
        render_asset::RenderAssets,
        render_resource::{
            AddressMode, Extent3d, ImageCopyTexture, Origin3d, SamplerDescriptor, TextureAspect,
            TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
            TextureViewDescriptor, TextureViewDimension,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::{BevyDefault, GpuImage},
    },
    utils::{HashMap, HashSet},
};

use crate::tilemap::map::{TilemapTextures, WaitForTextureUsageChange};

#[derive(Resource, Default)]
pub struct TilemapTexturesStorage {
    textures: HashMap<Handle<TilemapTextures>, GpuImage>,
    prepare_queue: HashSet<Handle<TilemapTextures>>,
    queue_queue: HashSet<Handle<TilemapTextures>>,
}

impl TilemapTexturesStorage {
    pub fn insert(&mut self, textures: Handle<TilemapTextures>) {
        #[cfg(not(feature = "atlas"))]
        self.prepare_queue.insert(textures);
        #[cfg(feature = "atlas")]
        self.queue_queue.insert(textures);
    }

    /// Try to get the processed texture array.
    pub fn get_texture(&self, handle: &Handle<TilemapTextures>) -> Option<&GpuImage> {
        self.textures.get(handle)
    }

    /// Prepare the texture, creating the texture array and translate images in `queue_texture` function.
    #[cfg(not(feature = "atlas"))]
    pub fn prepare_textures(
        &mut self,
        render_device: &RenderDevice,
        textures_assets: &RenderAssets<TilemapTextures>,
    ) {
        if self.prepare_queue.is_empty() {
            return;
        }

        let to_prepare = self.prepare_queue.drain().collect::<Vec<_>>();

        for textures_handle in &to_prepare {
            let Some(textures) = textures_assets.get(textures_handle) else {
                self.prepare_queue.insert(textures_handle.clone());
                continue;
            };

            textures.assert_uniform_tile_size();
            if textures.textures.is_empty() {
                continue;
            }

            let desc = &textures.textures[0].desc;
            let tile_count = textures.total_tile_count();

            let texture = render_device.create_texture(&TextureDescriptor {
                label: Some("tilemap_texture_array"),
                size: Extent3d {
                    width: desc.tile_size.x,
                    height: desc.tile_size.y,
                    depth_or_array_layers: tile_count,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::bevy_default(),
                usage: TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });

            let sampler = render_device.create_sampler(&SamplerDescriptor {
                label: Some("tilemap_texture_array_sampler"),
                address_mode_u: AddressMode::ClampToEdge,
                address_mode_v: AddressMode::ClampToEdge,
                address_mode_w: AddressMode::ClampToEdge,
                mag_filter: textures.filter_mode,
                min_filter: textures.filter_mode,
                mipmap_filter: textures.filter_mode,
                lod_min_clamp: 0.,
                lod_max_clamp: f32::MAX,
                compare: None,
                anisotropy_clamp: 1,
                border_color: None,
            });

            let texture_view = texture.create_view(&TextureViewDescriptor {
                label: Some("tilemap_texture_array_view"),
                format: Some(TextureFormat::bevy_default()),
                dimension: Some(TextureViewDimension::D2Array),
                aspect: TextureAspect::All,
                base_mip_level: 0,
                base_array_layer: 0,
                mip_level_count: None,
                array_layer_count: Some(tile_count),
            });

            let gpu_image = GpuImage {
                texture_format: texture.format(),
                mip_level_count: texture.mip_level_count(),
                texture,
                texture_view,
                sampler,
                size: desc.tile_size,
            };

            self.textures.insert(textures_handle.clone(), gpu_image);
            self.queue_queue.insert(textures_handle.clone());
        }
    }

    #[cfg(feature = "atlas")]
    pub fn prepare_textures(
        &mut self,
        render_device: &RenderDevice,
        textures_assets: &RenderAssets<TilemapTextures>,
    ) {
        if self.prepare_queue.is_empty() {
            return;
        }

        let to_prepare = self.prepare_queue.drain().collect::<Vec<_>>();

        for textures_handle in &to_prepare {
            let Some(textures) = textures_assets.get(textures_handle) else {
                self.prepare_queue.insert(textures_handle.clone());
                continue;
            };

            if textures.textures.is_empty() {
                continue;
            }

            let texture = render_device.create_texture(&TextureDescriptor {
                label: Some("tilemap_texture_array"),
                size: Extent3d {
                    width: textures.max_size.x,
                    height: textures.max_size.y,
                    depth_or_array_layers: textures.textures.len() as u32,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::bevy_default(),
                usage: TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });

            let sampler = render_device.create_sampler(&SamplerDescriptor {
                label: Some("tilemap_texture_array_sampler"),
                address_mode_u: AddressMode::ClampToEdge,
                address_mode_v: AddressMode::ClampToEdge,
                address_mode_w: AddressMode::ClampToEdge,
                mag_filter: textures.filter_mode,
                min_filter: textures.filter_mode,
                mipmap_filter: textures.filter_mode,
                lod_min_clamp: 0.,
                lod_max_clamp: f32::MAX,
                compare: None,
                anisotropy_clamp: 1,
                border_color: None,
            });

            let texture_view = texture.create_view(&TextureViewDescriptor {
                label: Some("tilemap_texture_array_view"),
                format: Some(TextureFormat::bevy_default()),
                dimension: Some(TextureViewDimension::D2Array),
                aspect: TextureAspect::All,
                base_mip_level: 0,
                base_array_layer: 0,
                mip_level_count: None,
                array_layer_count: Some(textures.textures.len() as u32),
            });

            let gpu_image = GpuImage {
                texture_format: texture.format(),
                mip_level_count: texture.mip_level_count(),
                texture,
                texture_view,
                sampler,
                size: textures.max_size,
            };

            self.textures.insert(textures_handle.clone(), gpu_image);
            self.queue_queue.insert(textures_handle.clone());
        }
    }

    /// Translate images to texture array.
    #[cfg(not(feature = "atlas"))]
    pub fn queue_textures(
        &mut self,
        render_device: &RenderDevice,
        render_queue: &RenderQueue,
        render_images: &RenderAssets<GpuImage>,
        textures_assets: &RenderAssets<TilemapTextures>,
    ) {
        if self.queue_queue.is_empty() {
            return;
        }

        let to_queue = self.queue_queue.drain().collect::<Vec<_>>();
        let mut command_encoder = render_device.create_command_encoder(&Default::default());

        for textures_handle in &to_queue {
            let Some(textures) = textures_assets.get(textures_handle) else {
                self.queue_queue.insert(textures_handle.clone());
                continue;
            };

            for (texture, start_index) in textures.iter_packed() {
                let image_handle = texture.handle();
                let desc = texture.desc();

                let Some(raw_gpu_image) = render_images.get(image_handle) else {
                    self.queue_queue.insert(textures_handle.clone());
                    continue;
                };

                if !raw_gpu_image
                    .texture
                    .usage()
                    .contains(TextureUsages::COPY_SRC)
                {
                    self.queue_queue.insert(textures_handle.clone());
                    continue;
                }

                let tile_count = desc.size / desc.tile_size;
                let array_gpu_image = self.textures.get(textures_handle).unwrap();

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
                                    z: index_x + index_y * tile_count.x + start_index,
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
            }
        }

        render_queue.submit(vec![command_encoder.finish()]);
    }

    #[cfg(feature = "atlas")]
    pub fn queue_textures(
        &mut self,
        render_device: &RenderDevice,
        render_queue: &RenderQueue,
        render_images: &mut RenderAssets<GpuImage>,
        textures_assets: &RenderAssets<TilemapTextures>,
    ) {
        if self.queue_queue.is_empty() {
            return;
        }

        let to_queue = self.queue_queue.drain().collect::<Vec<_>>();
        let mut command_encoder = render_device.create_command_encoder(&Default::default());

        for textures_handle in to_queue.into_iter() {
            let Some(textures) = textures_assets.get(&textures_handle) else {
                self.queue_queue.insert(textures_handle.clone());
                continue;
            };

            let Some(destination) = self.textures.get(&textures_handle) else {
                self.prepare_queue.insert(textures_handle);
                continue;
            };

            for (index, texture) in textures.textures.iter().enumerate() {
                let Some(source) = render_images.get_mut(texture.handle()) else {
                    self.queue_queue.insert(textures_handle.clone());
                    continue;
                };

                command_encoder.copy_texture_to_texture(
                    ImageCopyTexture {
                        texture: &source.texture,
                        mip_level: 0,
                        origin: Origin3d::ZERO,
                        aspect: TextureAspect::All,
                    },
                    ImageCopyTexture {
                        texture: &destination.texture,
                        mip_level: 0,
                        origin: Origin3d {
                            x: 0,
                            y: 0,
                            z: index as u32,
                        },
                        aspect: TextureAspect::All,
                    },
                    Extent3d {
                        width: texture.desc.size.x,
                        height: texture.desc.size.y,
                        depth_or_array_layers: 1,
                    },
                );
            }
        }

        render_queue.submit(vec![command_encoder.finish()]);
    }

    pub fn contains(&self, handle: &Handle<TilemapTextures>) -> bool {
        self.textures.contains_key(handle)
            || self.queue_queue.contains(handle)
            || self.prepare_queue.contains(handle)
    }
}

pub fn set_texture_usage(
    mut commands: Commands,
    tilemaps_query: Query<(Entity, &Handle<TilemapTextures>), With<WaitForTextureUsageChange>>,
    mut image_assets: ResMut<Assets<Image>>,
    textures_assets: Res<Assets<TilemapTextures>>,
) {
    // Bevy doesn't set the `COPY_SRC` usage for images by default, so we need to do it manually.
    tilemaps_query.iter().for_each(|(entity, textures)| {
        let Some(t) = &textures_assets.get(textures) else {
            panic!(
                "Failed to fetch the TilemapTexture, did you forget to add that on your tilemap?"
            )
        };

        for tex in &t.textures {
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
        }

        commands
            .entity(entity)
            .remove::<WaitForTextureUsageChange>();
    });
}
