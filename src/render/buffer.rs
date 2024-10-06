use bevy::{
    ecs::entity::EntityHashMap,
    math::{Mat2, Vec4},
    prelude::{Commands, Res, ResMut, Resource, Vec2},
    render::{
        render_resource::{BufferUsages, DynamicUniformBuffer, RawBufferVec, ShaderType},
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
#[derive(ShaderType)]
pub struct GpuTilemapTextureDescriptor {
    pub tile_count: bevy::math::UVec2,
    pub tile_uv_size: Vec2,
    pub uv_scale: Vec2,
}

#[cfg(feature = "atlas")]
#[derive(Resource, Default)]
pub struct TilemapTextureDescriptorBuffer(
    EntityHashMap<(
        StorageBuffer<Vec<GpuTilemapTextureDescriptor>>,
        Vec<GpuTilemapTextureDescriptor>,
    )>,
);

#[cfg(feature = "atlas")]
impl PerTilemapBuffersStorage<GpuTilemapTextureDescriptor> for TilemapTextureDescriptorBuffer {
    #[inline]
    fn get_mapper_mut(
        &mut self,
    ) -> &mut EntityHashMap<(
        StorageBuffer<Vec<GpuTilemapTextureDescriptor>>,
        Vec<GpuTilemapTextureDescriptor>,
    )> {
        &mut self.0
    }

    #[inline]
    fn get_mapper(
        &self,
    ) -> &EntityHashMap<(
        StorageBuffer<Vec<GpuTilemapTextureDescriptor>>,
        Vec<GpuTilemapTextureDescriptor>,
    )> {
        &self.0
    }
}

#[derive(Default)]
pub struct SharedTilemapBuffers {
    pub uniform: DynamicUniformBuffer<TilemapUniform>,
    pub indices: EntityHashMap<u32>,
}

pub struct UnsharedTilemapBuffers {
    pub animation: RawBufferVec<i32>,
}

impl Default for UnsharedTilemapBuffers {
    fn default() -> Self {
        Self {
            animation: RawBufferVec::new(BufferUsages::STORAGE),
        }
    }
}

#[derive(Resource, Default)]
pub struct TilemapBuffers {
    pub shared: SharedTilemapBuffers,
    pub unshared: EntityHashMap<UnsharedTilemapBuffers>,
}

pub fn prepare_tilemap_buffers(
    mut commands: Commands,
    tilemap_instances: Res<TilemapInstances>,
    mut tilemap_buffers: ResMut<TilemapBuffers>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    // TODO switch to Time<Real>
    time: Res<Time>,
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

        let unshared = tilemap_buffers.unshared.entry(*entity).or_default();
        if let Some(anim) = &tilemap.changed_animations {
            *unshared.animation.values_mut() = anim.0.clone();
            unshared
                .animation
                .write_buffer(&render_device, &render_queue);
        }
    }

    tilemap_buffers
        .shared
        .uniform
        .write_buffer(&render_device, &render_queue);
}
