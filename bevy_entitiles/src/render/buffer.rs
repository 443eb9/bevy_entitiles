use std::marker::PhantomData;

use bevy::{
    ecs::entity::Entity,
    math::{UVec4, Vec4},
    prelude::{Component, Resource, Vec2},
    render::{
        render_resource::{
            encase::{internal::WriteInto, rts_array::Length},
            BindingResource, DynamicStorageBuffer, DynamicUniformBuffer, ShaderType,
        },
        renderer::{RenderDevice, RenderQueue},
    },
    utils::EntityHashMap,
};

use crate::{MAX_ANIM_COUNT, MAX_ANIM_SEQ_LENGTH, MAX_ATLAS_COUNT};

use super::{extract::ExtractedTilemap, texture::TileUV};

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

#[derive(ShaderType, Clone, Copy, Default)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub struct TileAnimation {
    // because array stride must be a multiple of 16 bytes
    pub(crate) seq: [UVec4; MAX_ANIM_SEQ_LENGTH / 4],
    pub(crate) length: u32,
    pub(crate) fps: f32,
}

impl TileAnimation {
    pub fn new(sequence: Vec<u32>, fps: f32) -> Self {
        assert!(
            sequence.len() < MAX_ANIM_SEQ_LENGTH,
            "animation sequence is too long!, max is {}",
            MAX_ANIM_SEQ_LENGTH
        );

        let mut seq = [UVec4::ZERO; MAX_ANIM_SEQ_LENGTH / 4];
        let mut index = 0;
        let mut length = 0;
        while length + 4 < sequence.len() {
            seq[index] = UVec4::new(
                sequence[length],
                sequence[length + 1],
                sequence[length + 2],
                sequence[length + 3],
            );
            index += 1;
            length += 4;
        }

        for i in 0..4 {
            if length + i < sequence.len() {
                seq[index][i] = sequence[length + i];
            }
        }

        Self {
            seq,
            length: sequence.length() as u32,
            fps,
        }
    }
}

#[derive(ShaderType, Clone, Copy)]
pub struct TilemapUniform {
    pub translation: Vec2,
    pub tile_render_size: Vec2,
    pub tile_render_scale: Vec2,
    pub tile_slot_size: Vec2,
    pub pivot: Vec2,
    pub texture_size: Vec2,
    pub atlas_uvs: [Vec4; MAX_ATLAS_COUNT],
    pub anim_seqs: [TileAnimation; MAX_ANIM_COUNT],
    pub time: f32,
}

#[derive(Resource, Default)]
pub struct TilemapUniformBuffers {
    buffer: DynamicUniformBuffer<TilemapUniform>,
}

impl UniformBuffer<ExtractedTilemap, TilemapUniform> for TilemapUniformBuffers {
    /// Update the uniform buffer with the current tilemap uniforms.
    /// Returns the `TilemapUniform` component to be used in the tilemap render pass.
    fn insert(&mut self, extracted: &ExtractedTilemap) -> DynamicOffsetComponent<TilemapUniform> {
        let mut atlas_uvs = [Vec4::ZERO; MAX_ATLAS_COUNT];

        let (texture_size, tile_render_size) = if let Some(texture) = &extracted.texture {
            let desc = texture.desc();

            desc.tiles_uv.iter().enumerate().for_each(|(i, uv)| {
                atlas_uvs[i] = Vec4::new(uv.min.x, uv.min.y, uv.max.x, uv.max.y);
            });

            if desc.is_uniform {
                // if uniform, all the tiles are the same size as the first one.
                (desc.size.as_vec2(), desc.tiles_uv[0].render_size())
            } else {
                // if not, we need to use the tile_render_size data in vertex input.
                // so the UVec2::ZERO is just a placeholder.
                (desc.size.as_vec2(), Vec2::ZERO)
            }
        } else {
            // pure color mode
            (Vec2::ZERO, extracted.tile_slot_size)
        };

        let component = TilemapUniform {
            translation: extracted.translation,
            tile_render_size,
            tile_slot_size: extracted.tile_slot_size,
            tile_render_scale: extracted.tile_render_scale,
            pivot: extracted.pivot,
            texture_size,
            atlas_uvs,
            anim_seqs: extracted.anim_seqs,
            time: extracted.time,
        };

        let index = self.buffer.push(component);

        DynamicOffsetComponent::new(index)
    }

    #[inline]
    fn buffer(&mut self) -> &mut DynamicUniformBuffer<TilemapUniform> {
        &mut self.buffer
    }
}

#[derive(Resource, Default)]
pub struct TilemapStorageBuffers {
    atlas_uvs: EntityHashMap<Entity, DynamicStorageBuffer<Vec<Vec4>>>,
    anim_seqs: EntityHashMap<Entity, DynamicStorageBuffer<Vec<TileAnimation>>>,
}

impl TilemapStorageBuffers {
    pub fn insert_atlas_uvs(&mut self, tilemap: Entity, atlas_uvs: &Vec<TileUV>) {
        let mut buffer = DynamicStorageBuffer::default();
        buffer.push(atlas_uvs.iter().map(|uv| (*uv).into()).collect());
        self.atlas_uvs.insert(tilemap, buffer);
    }

    pub fn insert_anim_seqs(&mut self, tilemap: Entity, anim_seqs: &Vec<TileAnimation>) {
        let mut buffer = DynamicStorageBuffer::default();
        buffer.push(anim_seqs.clone());
        self.anim_seqs.insert(tilemap, buffer);
    }

    pub fn clear(&mut self) {
        // self.atlas_uvs.clear();
        // self.anim_seqs.clear();
    }

    pub fn atlas_uvs_binding(&self, tilemap: Entity) -> Option<BindingResource> {
        self.atlas_uvs
            .get(&tilemap)
            .map(|buffer| buffer.binding())
            .flatten()
    }

    pub fn anim_seqs_binding(&self, tilemap: Entity) -> Option<BindingResource> {
        self.anim_seqs
            .get(&tilemap)
            .map(|buffer| buffer.binding())
            .flatten()
    }

    pub fn write(&mut self, render_device: &RenderDevice, render_queue: &RenderQueue) {
        for buffer in self.atlas_uvs.values_mut() {
            buffer.write_buffer(render_device, render_queue);
        }

        for buffer in self.anim_seqs.values_mut() {
            buffer.write_buffer(render_device, render_queue);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_anim_import() {
        let anim = TileAnimation::new(vec![1, 2, 3, 4, 5, 6, 4, 2], 10.);
        assert_eq!(
            anim.seq,
            [
                UVec4::new(1, 2, 3, 4),
                UVec4::new(5, 6, 4, 2),
                UVec4::ZERO,
                UVec4::ZERO,
                UVec4::ZERO,
                UVec4::ZERO,
                UVec4::ZERO,
                UVec4::ZERO,
            ]
        );
        let anim = TileAnimation::new(vec![1, 2, 3, 4, 5, 6, 4, 2, 1], 10.);
        assert_eq!(
            anim.seq,
            [
                UVec4::new(1, 2, 3, 4),
                UVec4::new(5, 6, 4, 2),
                UVec4::new(1, 0, 0, 0),
                UVec4::ZERO,
                UVec4::ZERO,
                UVec4::ZERO,
                UVec4::ZERO,
                UVec4::ZERO,
            ]
        );
        let anim = TileAnimation::new(vec![1, 2, 3, 4, 5, 6, 4, 2, 1, 3], 10.);
        assert_eq!(
            anim.seq,
            [
                UVec4::new(1, 2, 3, 4),
                UVec4::new(5, 6, 4, 2),
                UVec4::new(1, 3, 0, 0),
                UVec4::ZERO,
                UVec4::ZERO,
                UVec4::ZERO,
                UVec4::ZERO,
                UVec4::ZERO,
            ]
        );
        let anim = TileAnimation::new(vec![1, 2, 3, 4, 5, 6, 4, 2, 1, 3, 6], 10.);
        assert_eq!(
            anim.seq,
            [
                UVec4::new(1, 2, 3, 4),
                UVec4::new(5, 6, 4, 2),
                UVec4::new(1, 3, 6, 0),
                UVec4::ZERO,
                UVec4::ZERO,
                UVec4::ZERO,
                UVec4::ZERO,
                UVec4::ZERO,
            ]
        );
    }
}
