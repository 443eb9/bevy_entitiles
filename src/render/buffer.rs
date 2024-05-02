use std::marker::PhantomData;

use bevy::{
    ecs::entity::{Entity, EntityHashMap},
    math::{Mat2, Vec4},
    prelude::{Component, Resource, Vec2},
    render::{
        render_resource::{
            encase::internal::WriteInto, BindingResource, DynamicUniformBuffer, ShaderSize,
            ShaderType, StorageBuffer,
        },
        renderer::{RenderDevice, RenderQueue},
    },
};

use crate::tilemap::map::TilemapType;

use super::{extract::ExtractedTilemap, material::TilemapMaterial};

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

#[derive(Resource)]
pub struct TilemapUniformBuffer<M: TilemapMaterial> {
    buffer: DynamicUniformBuffer<TilemapUniform>,
    _marker: PhantomData<M>,
}

impl<M: TilemapMaterial> Default for TilemapUniformBuffer<M> {
    fn default() -> Self {
        Self {
            buffer: DynamicUniformBuffer::default(),
            _marker: PhantomData,
        }
    }
}

impl<M: TilemapMaterial> UniformBuffer<(&ExtractedTilemap<M>, f32), TilemapUniform>
    for TilemapUniformBuffer<M>
{
    /// Update the uniform buffer with the current tilemap uniforms.
    /// Returns the `TilemapUniform` component to be used in the tilemap render pass.
    fn insert(
        &mut self,
        extracted: &(&ExtractedTilemap<M>, f32),
    ) -> DynamicOffsetComponent<TilemapUniform> {
        let (extracted, time) = (&extracted.0, extracted.1);

        DynamicOffsetComponent::new(self.buffer().push(&TilemapUniform {
            translation: extracted.transform.translation,
            rotation: extracted.transform.get_rotation_matrix(),
            tile_render_size: extracted.tile_render_size,
            slot_size: extracted.slot_size,
            pivot: extracted.tile_pivot,
            layer_opacities: extracted.layer_opacities,
            axis_dir: extracted.axis_flip.as_vec2(),
            hex_legs: match extracted.ty {
                TilemapType::Hexagonal(legs) => legs as f32,
                _ => 0.,
            },
            time,
        }))
    }

    #[inline]
    fn buffer(&mut self) -> &mut DynamicUniformBuffer<TilemapUniform> {
        &mut self.buffer
    }
}

#[cfg(feature = "atlas")]
#[derive(ShaderType)]
pub struct GpuTilemapTextureDescriptor {
    pub tile_count: bevy::math::UVec2,
    pub tile_uv_size: Vec2,
    pub uv_scale: Vec2,
}

#[derive(Resource, Default)]
pub struct TilemapAnimationBuffer(EntityHashMap<(StorageBuffer<Vec<i32>>, Vec<i32>)>);

impl PerTilemapBuffersStorage<i32> for TilemapAnimationBuffer {
    #[inline]
    fn get_mapper_mut(&mut self) -> &mut EntityHashMap<(StorageBuffer<Vec<i32>>, Vec<i32>)> {
        &mut self.0
    }

    #[inline]
    fn get_mapper(&self) -> &EntityHashMap<(StorageBuffer<Vec<i32>>, Vec<i32>)> {
        &self.0
    }
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
