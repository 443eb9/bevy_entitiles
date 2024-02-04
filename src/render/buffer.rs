use std::marker::PhantomData;

use bevy::{
    ecs::entity::Entity,
    math::{Mat2, Vec4},
    prelude::{Component, Resource, Vec2},
    render::{
        render_resource::{
            encase::internal::WriteInto, BindingResource, DynamicUniformBuffer, ShaderSize,
            ShaderType, StorageBuffer,
        },
        renderer::{RenderDevice, RenderQueue},
    },
    utils::EntityHashMap,
};

use crate::tilemap::map::TilemapType;

use super::extract::ExtractedTilemap;

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
    fn get_or_insert_buffer(&mut self, tilemap: Entity) -> &mut Vec<U> {
        &mut self.get_mapper().entry(tilemap).or_default().1
    }

    fn bindings(&mut self) -> EntityHashMap<Entity, BindingResource> {
        self.get_mapper()
            .iter()
            .filter_map(|(tilemap, (buffer, _))| buffer.binding().map(|res| (*tilemap, res)))
            .collect()
    }

    fn remove(&mut self, tilemap: Entity) {
        self.get_mapper().remove(&tilemap);
    }

    fn clear(&mut self) {
        self.get_mapper()
            .values_mut()
            .for_each(|(_, buffer)| buffer.clear());
    }

    fn write(&mut self, render_device: &RenderDevice, render_queue: &RenderQueue) {
        for (buffer, data) in self.get_mapper().values_mut() {
            buffer.set(std::mem::take(data));
            buffer.write_buffer(render_device, render_queue);
        }
    }

    fn get_mapper(&mut self) -> &mut EntityHashMap<Entity, (StorageBuffer<Vec<U>>, Vec<U>)>;
}

#[derive(ShaderType, Clone, Copy)]
pub struct TilemapUniform {
    pub translation: Vec2,
    pub rotation: Mat2,
    pub uv_rotation: u32,
    pub tile_render_size: Vec2,
    pub slot_size: Vec2,
    pub pivot: Vec2,
    pub layer_opacities: Vec4,
    pub axis_dir: Vec2,
    pub hex_legs: f32,
    pub time: f32,
    #[cfg(feature = "atlas")]
    pub texture_tiled_size: bevy::math::IVec2,
    #[cfg(feature = "atlas")]
    pub tile_uv_size: Vec2,
}

#[derive(Resource, Default)]
pub struct TilemapUniformBuffer(DynamicUniformBuffer<TilemapUniform>);

impl UniformBuffer<(&ExtractedTilemap, f32), TilemapUniform> for TilemapUniformBuffer {
    /// Update the uniform buffer with the current tilemap uniforms.
    /// Returns the `TilemapUniform` component to be used in the tilemap render pass.
    fn insert(
        &mut self,
        extracted: &(&ExtractedTilemap, f32),
    ) -> DynamicOffsetComponent<TilemapUniform> {
        let (extracted, time) = (&extracted.0, extracted.1);

        let uv_rotation = {
            if let Some(tex) = extracted.texture.as_ref() {
                tex.rotation as u32 / 90
            } else {
                0
            }
        };

        #[cfg(feature = "atlas")]
        let (texture_tiled_size, tile_uv_size) = {
            if let Some(tex) = extracted.texture.as_ref() {
                (
                    (tex.desc.size / tex.desc.tile_size).as_ivec2(),
                    tex.desc.tile_size.as_vec2() / tex.desc.size.as_vec2(),
                )
            } else {
                (bevy::math::IVec2::ZERO, Vec2::ZERO)
            }
        };

        DynamicOffsetComponent::new(self.buffer().push(TilemapUniform {
            translation: extracted.transform.translation,
            rotation: extracted.transform.get_rotation_matrix(),
            uv_rotation,
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
            #[cfg(feature = "atlas")]
            texture_tiled_size,
            #[cfg(feature = "atlas")]
            tile_uv_size,
        }))
    }

    #[inline]
    fn buffer(&mut self) -> &mut DynamicUniformBuffer<TilemapUniform> {
        &mut self.0
    }
}

#[derive(Resource, Default)]
pub struct TilemapStorageBuffers(EntityHashMap<Entity, (StorageBuffer<Vec<i32>>, Vec<i32>)>);

impl PerTilemapBuffersStorage<i32> for TilemapStorageBuffers {
    fn get_mapper(&mut self) -> &mut EntityHashMap<Entity, (StorageBuffer<Vec<i32>>, Vec<i32>)> {
        &mut self.0
    }
}
