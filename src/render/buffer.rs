use std::marker::PhantomData;

use bevy::{
    ecs::entity::{Entity, EntityHashMap},
    math::{Mat2, Vec4},
    prelude::{Commands, Component, IntoSystem, Query, Res, ResMut, Resource, Vec2, With},
    render::{
        render_resource::{
            encase::internal::WriteInto, BindingResource, BufferUsages, BufferVec,
            DynamicUniformBuffer, RawBufferVec, ShaderSize, ShaderType, StorageBuffer,
        },
        renderer::{RenderDevice, RenderQueue},
    },
    time::{Real, Time},
};

use crate::{
    render::{
        extract::{ExtractedTilemap, TilemapInstance},
        material::TilemapMaterial,
        resources::TilemapInstances,
    },
    tilemap::map::TilemapType,
};

pub trait UniformBuffer<E, U: ShaderType + WriteInto + 'static> {
    fn insert(&mut self, extracted: &E) -> DynamicOffsetComponent<U>;

    fn binding(&mut self) -> Option<BindingResource> {
        self.buffer().binding()
    }

    fn clear(&mut self) {
        self.buffer().clear();
    }

    fn write(&mut self, render_device: &RenderDevice, render_queue: &RenderQueue) {
        self.buffer().write_buffer(render_device, render_queue);
    }

    fn buffer(&mut self) -> &mut DynamicUniformBuffer<U>;
}

#[derive(Component)]
pub struct DynamicOffsetComponent<T>
where
    T: ShaderType,
{
    index: u32,
    _marker: PhantomData<T>,
}

impl<T: ShaderType> DynamicOffsetComponent<T> {
    pub fn new(index: u32) -> Self {
        Self {
            index,
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn index(&self) -> u32 {
        self.index
    }
}

pub trait PerTilemapBuffersStorage<U: ShaderType + WriteInto + ShaderSize + 'static> {
    #[inline]
    fn get_or_insert_buffer(&mut self, tilemap: Entity) -> &mut Vec<U> {
        &mut self.get_mapper_mut().entry(tilemap).or_default().1
    }

    fn bindings(&mut self) -> EntityHashMap<BindingResource> {
        self.get_mapper_mut()
            .iter()
            .filter_map(|(tilemap, (buffer, _))| buffer.binding().map(|res| (*tilemap, res)))
            .collect()
    }

    #[inline]
    fn remove(&mut self, tilemap: Entity) {
        self.get_mapper_mut().remove(&tilemap);
    }

    #[inline]
    fn clear(&mut self) {
        self.get_mapper_mut()
            .values_mut()
            .for_each(|(_, buffer)| buffer.clear());
    }

    fn write(&mut self, render_device: &RenderDevice, render_queue: &RenderQueue) {
        for (buffer, data) in self.get_mapper_mut().values_mut() {
            buffer.set(std::mem::take(data));
            buffer.write_buffer(render_device, render_queue);
        }
    }

    #[inline]
    fn len(&self) -> usize {
        self.get_mapper().len()
    }

    fn get_mapper_mut(&mut self) -> &mut EntityHashMap<(StorageBuffer<Vec<U>>, Vec<U>)>;

    fn get_mapper(&self) -> &EntityHashMap<(StorageBuffer<Vec<U>>, Vec<U>)>;
}

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

pub fn prepare_tilemap_buffers<M: TilemapMaterial>(
    mut commands: Commands,
    extracted_tilemaps: Query<Entity, With<TilemapInstance>>,
    tilemap_instances: Res<TilemapInstances<M>>,
    mut tilemap_buffers: ResMut<TilemapBuffers>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    // TODO switch to Time<Real>
    time: Res<Time>,
) {
    tilemap_buffers.shared.uniform.clear();

    for tilemap in extracted_tilemaps
        .iter()
        .filter_map(|e| tilemap_instances.0.get(&e))
    {
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
        commands
            .entity(tilemap.id)
            .insert(DynamicOffsetComponent::<TilemapUniform>::new(index));

        let unshared = tilemap_buffers.unshared.entry(tilemap.id).or_default();
        if let Some(anim) = &tilemap.animations {
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
