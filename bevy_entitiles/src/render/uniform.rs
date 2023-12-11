use bevy::{
    math::{UVec4, Vec4},
    prelude::{Component, Resource, Vec2},
    render::{
        render_resource::{
            encase::{internal::WriteInto, rts_array::Length},
            BindingResource, DynamicUniformBuffer, ShaderType,
        },
        renderer::{RenderDevice, RenderQueue},
    },
};

use crate::{MAX_ANIM_COUNT, MAX_ANIM_SEQ_LENGTH, MAX_ATLAS_COUNT};

use super::extract::ExtractedTilemap;

pub trait UniformsStorage<E, U: ShaderType + WriteInto + 'static> {
    /// Update the uniform buffer with the current uniforms.
    /// Returns the `U` component to be used in the render pass.
    fn insert(&mut self, extracted: &E) -> DynamicUniformComponent<U>;

    /// Get the binding resource for the uniform buffer.
    fn binding(&mut self) -> Option<BindingResource> {
        self.buffer().binding()
    }

    /// Clear the uniform buffer.
    fn clear(&mut self) {
        self.buffer().clear();
    }

    /// Write the uniform buffer to the GPU.
    fn write(&mut self, render_device: &RenderDevice, render_queue: &RenderQueue) {
        self.buffer().write_buffer(render_device, render_queue);
    }

    fn buffer(&mut self) -> &mut DynamicUniformBuffer<U>;
}

#[derive(Component)]
pub struct DynamicUniformComponent<T>
where
    T: ShaderType,
{
    pub index: u32,
    pub component: T,
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
pub struct TilemapUniformsStorage {
    buffer: DynamicUniformBuffer<TilemapUniform>,
}

impl UniformsStorage<ExtractedTilemap, TilemapUniform> for TilemapUniformsStorage {
    /// Update the uniform buffer with the current tilemap uniforms.
    /// Returns the `TilemapUniform` component to be used in the tilemap render pass.
    fn insert(&mut self, extracted: &ExtractedTilemap) -> DynamicUniformComponent<TilemapUniform> {
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

        DynamicUniformComponent { index, component }
    }

    #[inline]
    fn buffer(&mut self) -> &mut DynamicUniformBuffer<TilemapUniform> {
        &mut self.buffer
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_anim_import() {
        let anim = TileAnimation::new(vec![1, 2, 3, 4, 5, 6, 4, 2], 10.);
        assert_eq!(anim.seq, 
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
        assert_eq!(anim.seq, 
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
        assert_eq!(anim.seq, 
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
        assert_eq!(anim.seq, 
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
