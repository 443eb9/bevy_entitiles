use bevy::{
    asset::{AssetId, Assets, Handle},
    ecs::{
        entity::Entity,
        query::With,
        system::{Commands, Query, Res, ResMut, Resource},
    },
    prelude::Image,
    render::{
        render_asset::RenderAssets,
        render_resource::{AddressMode, SamplerDescriptor, TextureUsages},
        renderer::RenderDevice,
        texture::GpuImage,
    },
    utils::{HashMap, HashSet},
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

use crate::tilemap::map::WaitForTextureUsageChange;

use super::material::{StandardTilemapMaterial, StandardTilemapMaterialInstances};

#[derive(Resource, Default)]
pub struct TilemapTexturesStorage {
    textures: HashMap<AssetId<StandardTilemapMaterial>, GpuImage>,
    prepare_queue: HashSet<AssetId<StandardTilemapMaterial>>,
    queue_queue: HashSet<AssetId<StandardTilemapMaterial>>,
}

impl TilemapTexturesStorage {
    pub fn insert(&mut self, material: AssetId<StandardTilemapMaterial>) {
        #[cfg(not(feature = "atlas"))]
        self.prepare_queue.insert(material);
        #[cfg(feature = "atlas")]
        self.queue_queue.insert(material);
    }

    /// Try to get the processed texture array.
    pub fn get_texture(&self, material: &AssetId<StandardTilemapMaterial>) -> Option<&GpuImage> {
        self.textures.get(material)
    }

    /// Prepare the texture, creating the texture array and translate images in `queue_texture` function.
    #[cfg(not(feature = "atlas"))]
    pub fn prepare_textures(
        &mut self,
        render_device: &RenderDevice,
        materials: &StandardTilemapMaterialInstances,
    ) {
        if self.prepare_queue.is_empty() {
            return;
        }

        let to_prepare = self.prepare_queue.drain().collect::<Vec<_>>();

        for material_handle in to_prepare.iter() {
            let Some(material) = materials.get(material_handle) else {
                continue;
            };
            let tilemap_texture = material.texture.as_ref().unwrap();

            let tile_count = tilemap_texture.desc.size / tilemap_texture.desc.tile_size;

            let texture = render_device.create_texture(&TextureDescriptor {
                label: Some("tilemap_texture_array"),
                size: Extent3d {
                    width: tilemap_texture.desc.tile_size.x,
                    height: tilemap_texture.desc.tile_size.y,
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
                mag_filter: tilemap_texture.desc.filter_mode,
                min_filter: tilemap_texture.desc.filter_mode,
                mipmap_filter: tilemap_texture.desc.filter_mode,
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
                size: Vec2::new(
                    tilemap_texture.desc.tile_size.x as f32,
                    tilemap_texture.desc.tile_size.y as f32,
                ),
            };

            self.textures.insert(*material_handle, gpu_image);
            self.queue_queue.insert(*material_handle);
        }
    }

    /// Translate images to texture array.
    #[cfg(not(feature = "atlas"))]
    pub fn queue_textures(
        &mut self,
        render_device: &RenderDevice,
        render_queue: &RenderQueue,
        render_images: &RenderAssets<Image>,
        materials: &StandardTilemapMaterialInstances,
    ) {
        if self.queue_queue.is_empty() {
            return;
        }

        let to_queue = self.queue_queue.drain().collect::<Vec<_>>();

        for material_handle in to_queue.iter() {
            let Some(texture) = materials
                .get(material_handle)
                .and_then(|m| m.texture.as_ref())
            else {
                continue;
            };

            let Some(raw_gpu_image) = render_images.get(&texture.texture) else {
                self.queue_queue.insert(*material_handle);
                continue;
            };

            if !raw_gpu_image
                .texture
                .usage()
                .contains(TextureUsages::COPY_SRC)
            {
                self.queue_queue.insert(*material_handle);
                continue;
            }

            let tile_count = texture.desc.size / texture.desc.tile_size;
            let array_gpu_image = self.textures.get(material_handle).unwrap();
            let mut command_encoder = render_device.create_command_encoder(&Default::default());

            for index_y in 0..tile_count.y {
                for index_x in 0..tile_count.x {
                    command_encoder.copy_texture_to_texture(
                        ImageCopyTexture {
                            texture: &raw_gpu_image.texture,
                            mip_level: 0,
                            origin: Origin3d {
                                x: index_x * texture.desc.tile_size.x,
                                y: index_y * texture.desc.tile_size.y,
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
                            width: texture.desc.tile_size.x,
                            height: texture.desc.tile_size.y,
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
        materials: &StandardTilemapMaterialInstances,
        render_images: &mut RenderAssets<Image>,
    ) {
        if self.queue_queue.is_empty() {
            return;
        }

        let to_queue = self.queue_queue.drain().collect::<Vec<_>>();

        for material_handle in to_queue.into_iter() {
            let Some(material) = materials.get(&material_handle) else {
                self.queue_queue.insert(material_handle);
                continue;
            };

            let Some(texture) = &material.texture else {
                continue;
            };

            let Some(gpu_texture) = render_images.get_mut(&texture.texture) else {
                self.queue_queue.insert(material_handle);
                continue;
            };

            let sampler = render_device.create_sampler(&SamplerDescriptor {
                label: Some("tilemap_texture_atlas_sampler"),
                address_mode_u: AddressMode::ClampToEdge,
                address_mode_v: AddressMode::ClampToEdge,
                address_mode_w: AddressMode::ClampToEdge,
                mag_filter: texture.desc.filter_mode,
                min_filter: texture.desc.filter_mode,
                mipmap_filter: texture.desc.filter_mode,
                lod_min_clamp: 0.,
                lod_max_clamp: f32::MAX,
                compare: None,
                anisotropy_clamp: 1,
                border_color: None,
            });

            gpu_texture.sampler = sampler;
            self.textures
                .insert(material_handle.clone(), gpu_texture.clone());
        }
    }

    pub fn contains(&self, handle: &AssetId<StandardTilemapMaterial>) -> bool {
        self.textures.contains_key(handle)
            || self.queue_queue.contains(handle)
            || self.prepare_queue.contains(handle)
    }
}

pub fn set_texture_usage(
    mut commands: Commands,
    tilemaps_query: Query<
        (Entity, &Handle<StandardTilemapMaterial>),
        With<WaitForTextureUsageChange>,
    >,
    materials: Res<Assets<StandardTilemapMaterial>>,
    mut image_assets: ResMut<Assets<Image>>,
) {
    // Bevy doesn't set the `COPY_SRC` usage for images by default, so we need to do it manually.
    tilemaps_query.iter().for_each(|(entity, mat)| {
        let Some(tex) = &materials.get(mat).and_then(|m| m.texture.clone()) else {
            return;
        };

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
