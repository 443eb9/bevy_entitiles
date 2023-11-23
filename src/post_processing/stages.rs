use bevy::{
    core_pipeline::core_2d::Transparent2d,
    ecs::{
        entity::Entity,
        query::{Changed, Or},
        system::{
            lifetimeless::{Read, SRes},
            Commands, Query, Res, ResMut,
        },
    },
    math::Vec2,
    render::{
        render_asset::RenderAssets,
        render_phase::{RenderCommand, RenderCommandResult},
        render_resource::{
            AddressMode, BindGroupEntry, BindingResource, Extent3d, FilterMode, SamplerDescriptor,
            TextureAspect, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
            TextureViewDescriptor, TextureViewDimension,
        },
        renderer::RenderDevice,
        texture::{GpuImage, Image},
        view::ExtractedWindows,
        Extract,
    },
};

use crate::{
    render::{
        extract::ExtractedHeightTilemap, pipeline::EntiTilesPipeline,
        texture::TilemapTexturesStorage,
    },
    tilemap::{
        post_processing::height::HeightTilemap,
        tile::{Tile, TileAnimation},
    },
};

use super::{PostProcessingBindGroups, PostProcessingSettings, PostProcessingTextures};

pub fn extract_height_maps(
    mut commands: Commands,
    height_tilemaps: Extract<Query<(Entity, &HeightTilemap)>>,
) {
    let mut extracted_height_tilemaps = vec![];
    for (entity, height_map) in height_tilemaps.iter() {
        extracted_height_tilemaps.push((
            entity,
            ExtractedHeightTilemap {
                height_texture: height_map.height_texture.clone(),
            },
        ));
    }
    commands.insert_or_spawn_batch(extracted_height_tilemaps);
}

pub fn prepare_post_processing(
    render_device: Res<RenderDevice>,
    extracted_height_tilemaps: Query<&ExtractedHeightTilemap>,
    mut textures_storage: ResMut<TilemapTexturesStorage>,
    render_images: Res<RenderAssets<Image>>,
) {
    for tilemap in extracted_height_tilemaps.iter() {
        if textures_storage.contains(&tilemap.height_texture.handle()) {
            continue;
        }

        let tex = &tilemap.height_texture;
        textures_storage.insert(
            tex.clone_weak(),
            render_images.get(tex.handle()).unwrap().clone(),
            render_device.create_sampler(&SamplerDescriptor {
                label: Some("tilemap_height_texture_sampler"),
                address_mode_u: AddressMode::ClampToEdge,
                address_mode_v: AddressMode::ClampToEdge,
                address_mode_w: AddressMode::ClampToEdge,
                mag_filter: tex.desc.filter_mode,
                min_filter: tex.desc.filter_mode,
                mipmap_filter: tex.desc.filter_mode,
                lod_min_clamp: 0.,
                lod_max_clamp: f32::MAX,
                compare: None,
                anisotropy_clamp: 1,
                border_color: None,
            }),
        );
    }
}

pub fn prepare_post_processing_textures(
    render_device: Res<RenderDevice>,
    settings: Res<PostProcessingSettings>,
    mut textures: ResMut<PostProcessingTextures>,
    windows: Res<ExtractedWindows>,
) {
    let window = windows.windows.get(&windows.primary.unwrap()).unwrap();
    let size = Vec2 {
        x: window.physical_width as f32,
        y: window.physical_height as f32,
    };

    if let Some(tex) = textures.screen_height_gpu_image.as_ref() {
        if tex.size == size && tex.texture_format == TextureFormat::Rgba8Unorm {
            return;
        }
    }

    let texture = render_device.create_texture(&TextureDescriptor {
        label: Some("screen_height_texture"),
        size: Extent3d {
            width: window.physical_width,
            height: window.physical_height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8Unorm,
        usage: TextureUsages::STORAGE_BINDING,
        view_formats: &[],
    });

    let sampler = render_device.create_sampler(&SamplerDescriptor {
        label: Some("screen_height_texture_sampler"),
        address_mode_u: AddressMode::ClampToEdge,
        address_mode_v: AddressMode::ClampToEdge,
        address_mode_w: AddressMode::ClampToEdge,
        mag_filter: settings.filter_mode,
        min_filter: settings.filter_mode,
        mipmap_filter: settings.filter_mode,
        lod_min_clamp: 0.,
        lod_max_clamp: f32::MAX,
        compare: None,
        anisotropy_clamp: 1,
        border_color: None,
    });

    let texture_view = texture.create_view(&TextureViewDescriptor {
        label: Some("screen_height_texture_view"),
        format: Some(TextureFormat::Rgba8Unorm),
        dimension: Some(TextureViewDimension::D2),
        aspect: TextureAspect::All,
        base_mip_level: 0,
        base_array_layer: 0,
        mip_level_count: None,
        array_layer_count: None,
    });

    textures.screen_height_gpu_image = Some(GpuImage {
        texture_format: texture.format(),
        mip_level_count: texture.mip_level_count(),
        texture,
        texture_view,
        sampler,
        size,
    });

    textures.screen_color_texture_sampler =
        Some(render_device.create_sampler(&SamplerDescriptor {
            label: Some("screen_color_texture_sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Linear,
            lod_min_clamp: 0.,
            lod_max_clamp: f32::MAX,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        }));
}

pub struct SetGlobalsUniformBindGroup<const I: usize>;
impl<const I: usize> RenderCommand<Transparent2d> for SetGlobalsUniformBindGroup<I> {
    type Param = SRes<PostProcessingBindGroups>;

    type ViewWorldQuery = ();

    type ItemWorldQuery = ();

    fn render<'w>(
        _item: &Transparent2d,
        _view: bevy::ecs::query::ROQueryItem<'w, Self::ViewWorldQuery>,
        _entity: bevy::ecs::query::ROQueryItem<'w, Self::ItemWorldQuery>,
        bind_groups: bevy::ecs::system::SystemParamItem<'w, '_, Self::Param>,
        pass: &mut bevy::render::render_phase::TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        pass.set_bind_group(
            I,
            bind_groups
                .into_inner()
                .uniforms_bind_group
                .as_ref()
                .unwrap(),
            &[],
        );

        RenderCommandResult::Success
    }
}

pub struct SetTilemapHeightTextureBindGroup<const I: usize>;
impl<const I: usize> RenderCommand<Transparent2d> for SetTilemapHeightTextureBindGroup<I> {
    type Param = SRes<PostProcessingBindGroups>;

    type ViewWorldQuery = ();

    type ItemWorldQuery = Read<ExtractedHeightTilemap>;

    fn render<'w>(
        _item: &Transparent2d,
        _view: bevy::ecs::query::ROQueryItem<'w, Self::ViewWorldQuery>,
        tilemap: bevy::ecs::query::ROQueryItem<'w, Self::ItemWorldQuery>,
        bind_groups: bevy::ecs::system::SystemParamItem<'w, '_, Self::Param>,
        pass: &mut bevy::render::render_phase::TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        pass.set_bind_group(
            I,
            bind_groups
                .into_inner()
                .height_texture_bind_groups
                .get(tilemap.height_texture.handle())
                .unwrap(),
            &[],
        );

        RenderCommandResult::Success
    }
}

pub struct SetScreenHeightTextureBindGroup<const I: usize>;
impl<const I: usize> RenderCommand<Transparent2d> for SetScreenHeightTextureBindGroup<I> {
    type Param = SRes<PostProcessingBindGroups>;

    type ViewWorldQuery = ();

    type ItemWorldQuery = ();

    fn render<'w>(
        _item: &Transparent2d,
        _view: bevy::ecs::query::ROQueryItem<'w, Self::ViewWorldQuery>,
        _entity: bevy::ecs::query::ROQueryItem<'w, Self::ItemWorldQuery>,
        post_processing_bind_groups: bevy::ecs::system::SystemParamItem<'w, '_, Self::Param>,
        pass: &mut bevy::render::render_phase::TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        pass.set_bind_group(
            I,
            post_processing_bind_groups
                .into_inner()
                .screen_height_texture_bind_group
                .as_ref()
                .unwrap(),
            &[],
        );

        RenderCommandResult::Success
    }
}
