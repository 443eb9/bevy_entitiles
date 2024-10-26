use bevy::{
    ecs::entity::EntityHashMap,
    math::{Mat2, Vec4},
    prelude::{Res, ResMut, Resource, Vec2},
    render::{
        render_resource::{DynamicUniformBuffer, GpuArrayBuffer, ShaderType},
        renderer::{RenderDevice, RenderQueue},
    },
    time::Time,
};

use crate::{render::extract::TilemapInstances, tilemap::map::TilemapType};

#[derive(ShaderType, Clone, Copy)]
pub struct TilemapUniform {
    pub translation: Vec2,
    pub rotation: Mat2,
    pub tile_render_size: Vec2,
    pub slot_size: Vec2,
    pub pivot: Vec2,
    pub layer_opacities: Vec4,
    pub axis_dir: Vec2,
    pub hex_legs: f32,
    pub time: f32,
}

#[cfg(feature = "atlas")]
#[derive(ShaderType, Default, Clone)]
pub struct GpuTilemapTextureDescriptor {
    pub tile_uv_size: Vec2,
    pub uv_scale: Vec2,
    pub tile_count: bevy::math::UVec2,
    #[cfg(target_arch = "wasm32")]
    pub _padding: [u32; 2],
}

#[derive(Default)]
pub struct SharedTilemapBuffers {
    pub uniform: DynamicUniformBuffer<TilemapUniform>,
    pub indices: EntityHashMap<u32>,
}

pub struct UnsharedTilemapBuffers {
    pub animation: GpuArrayBuffer<i32>,
    #[cfg(feature = "atlas")]
    pub texture_desc: GpuArrayBuffer<GpuTilemapTextureDescriptor>,
}

impl UnsharedTilemapBuffers {
    pub fn new(render_device: &RenderDevice) -> Self {
        Self {
            animation: GpuArrayBuffer::new(render_device),
            #[cfg(feature = "atlas")]
            texture_desc: GpuArrayBuffer::new(render_device),
        }
    }
}

#[derive(Resource, Default)]
pub struct TilemapBuffers {
    pub shared: SharedTilemapBuffers,
    pub unshared: EntityHashMap<UnsharedTilemapBuffers>,
}

pub fn prepare_tilemap_buffers(
    tilemap_instances: Res<TilemapInstances>,
    mut tilemap_buffers: ResMut<TilemapBuffers>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    time: Res<Time>,
    #[cfg(feature = "atlas")] textures_assets: Res<
        bevy::render::render_asset::RenderAssets<crate::tilemap::map::TilemapTextures>,
    >,
) {
    tilemap_buffers.shared.uniform.clear();

    for (entity, tilemap) in tilemap_instances.iter() {
        let index = tilemap_buffers.shared.uniform.push(&TilemapUniform {
            translation: tilemap.transform.translation,
            rotation: tilemap.transform.get_rotation_matrix(),
            tile_render_size: tilemap.tile_render_size,
            slot_size: tilemap.slot_size,
            pivot: tilemap.tile_pivot,
            layer_opacities: tilemap.layer_opacities,
            axis_dir: tilemap.axis_flip.as_vec2(),
            hex_legs: match tilemap.ty {
                TilemapType::Hexagonal(legs) => legs as f32,
                _ => 0.,
            },
            time: time.elapsed_seconds(),
        });
        tilemap_buffers.shared.indices.insert(*entity, index);

        let unshared = tilemap_buffers
            .unshared
            .entry(*entity)
            .or_insert_with(|| UnsharedTilemapBuffers::new(&render_device));
        if let Some(anim) = &tilemap.changed_animations {
            for data in &anim.0 {
                unshared.animation.push(*data);
            }
            unshared
                .animation
                .write_buffer(&render_device, &render_queue);
        }

        #[cfg(feature = "atlas")]
        if let Some(handle) = &tilemap.texture {
            if let Some(textures) = textures_assets.get(handle) {
                unshared.texture_desc.clear();

                for (i, t) in textures.textures.iter().enumerate() {
                    unshared.texture_desc.push(GpuTilemapTextureDescriptor {
                        tile_count: t.desc.size / t.desc.tile_size,
                        tile_uv_size: t.desc.tile_size.as_vec2() / t.desc.size.as_vec2(),
                        uv_scale: textures.uv_scales[i],
                        ..Default::default()
                    });
                }

                unshared
                    .texture_desc
                    .write_buffer(&render_device, &render_queue);
            }
        }
    }

    tilemap_buffers
        .shared
        .uniform
        .write_buffer(&render_device, &render_queue);
}
