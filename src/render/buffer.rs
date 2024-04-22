use std::{hash::Hash, marker::PhantomData};

use bevy::{
    asset::AssetId,
    ecs::entity::{Entity, EntityHashMap},
    math::{Mat2, UVec2, Vec4},
    prelude::{Component, Resource, Vec2},
    render::{
        render_resource::{
            encase::internal::WriteInto, BindingResource, DynamicUniformBuffer, ShaderSize,
            ShaderType, StorageBuffer,
        },
        renderer::{RenderDevice, RenderQueue},
    },
    utils::HashMap,
};

use crate::tilemap::map::{TilemapTexture, TilemapTextureDescriptor, TilemapType};

use super::{extract::ExtractedTilemap, material::StandardTilemapMaterial};

pub trait UniformBuffer<E, U: ShaderType + WriteInto + 'static, I: PartialEq + Eq + Hash> {
    fn insert(&mut self, extracted: &E, ident: I);

    fn binding(&self) -> Option<BindingResource> {
        self.buffer().binding()
    }

    fn clear(&mut self) {
        self.buffer_mut().clear();
    }

    fn write(&mut self, render_device: &RenderDevice, render_queue: &RenderQueue) {
        self.buffer_mut().write_buffer(render_device, render_queue);
    }

    fn buffer(&self) -> &DynamicUniformBuffer<U>;

    fn buffer_mut(&mut self) -> &mut DynamicUniformBuffer<U>;
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

    fn bindings(&mut self) -> EntityHashMap<BindingResource> {
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

    fn get_mapper(&mut self) -> &mut EntityHashMap<(StorageBuffer<Vec<U>>, Vec<U>)>;
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

#[derive(ShaderType, Clone, Copy)]
pub struct StandardMaterialUniform {
    pub num_tiles: UVec2,
    pub tile_uv_size: Vec2,
    pub uv_rotation: u32,
}

#[derive(Resource)]
pub struct TilemapUniformBuffer {
    tilemap: DynamicUniformBuffer<TilemapUniform>,
    pub offsets: EntityHashMap<DynamicOffsetComponent<TilemapUniform>>,
}

impl Default for TilemapUniformBuffer {
    fn default() -> Self {
        Self {
            tilemap: DynamicUniformBuffer::default(),
            offsets: EntityHashMap::default(),
        }
    }
}

impl UniformBuffer<(&ExtractedTilemap, f32), TilemapUniform, Entity> for TilemapUniformBuffer {
    /// Update the uniform buffer with the current tilemap uniforms.
    /// Returns the `TilemapUniform` component to be used in the tilemap render pass.
    fn insert(&mut self, extracted: &(&ExtractedTilemap, f32), ident: Entity) {
        let (extracted, time) = (&extracted.0, extracted.1);

        let offset = self.buffer_mut().push(&TilemapUniform {
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
        });

        self.offsets
            .insert(ident, DynamicOffsetComponent::new(offset));
    }

    #[inline]
    fn buffer(&self) -> &DynamicUniformBuffer<TilemapUniform> {
        &self.tilemap
    }

    #[inline]
    fn buffer_mut(&mut self) -> &mut DynamicUniformBuffer<TilemapUniform> {
        &mut self.tilemap
    }
}

#[derive(Resource, Default)]
pub struct StandardMaterialUniformBuffer {
    materials: DynamicUniformBuffer<StandardMaterialUniform>,
    pub offsets:
        HashMap<AssetId<StandardTilemapMaterial>, DynamicOffsetComponent<StandardMaterialUniform>>,
}

impl
    UniformBuffer<
        StandardTilemapMaterial,
        StandardMaterialUniform,
        AssetId<StandardTilemapMaterial>,
    > for StandardMaterialUniformBuffer
{
    fn insert(
        &mut self,
        extracted: &StandardTilemapMaterial,
        ident: AssetId<StandardTilemapMaterial>,
    ) {
        let texture = extracted.texture.clone().unwrap_or_else(|| TilemapTexture {
            desc: TilemapTextureDescriptor {
                size: UVec2::ONE,
                tile_size: UVec2::ONE,
                ..Default::default()
            },
            ..Default::default()
        });

        let offset = self.materials.push(&StandardMaterialUniform {
            num_tiles: texture.desc.size / texture.desc.tile_size,
            tile_uv_size: texture.desc.tile_size.as_vec2() / texture.desc.size.as_vec2(),
            uv_rotation: texture.rotation as u32,
        });

        self.offsets
            .insert(ident, DynamicOffsetComponent::new(offset));
    }

    #[inline]
    fn buffer(&self) -> &DynamicUniformBuffer<StandardMaterialUniform> {
        &self.materials
    }

    #[inline]
    fn buffer_mut(&mut self) -> &mut DynamicUniformBuffer<StandardMaterialUniform> {
        &mut self.materials
    }
}

#[derive(Resource, Default)]
pub struct TilemapStorageBuffers(EntityHashMap<(StorageBuffer<Vec<i32>>, Vec<i32>)>);

impl PerTilemapBuffersStorage<i32> for TilemapStorageBuffers {
    fn get_mapper(&mut self) -> &mut EntityHashMap<(StorageBuffer<Vec<i32>>, Vec<i32>)> {
        &mut self.0
    }
}
