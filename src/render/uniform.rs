use bevy::{
    prelude::{Component, Resource, Vec2},
    render::{
        render_resource::{BindingResource, DynamicUniformBuffer, ShaderType},
        renderer::{RenderDevice, RenderQueue},
    },
};

use super::extract::ExtractedTilemap;

#[derive(Component)]
pub struct DynamicUniformComponent<T>
where
    T: ShaderType,
{
    pub index: u32,
    pub component: T,
}

#[derive(ShaderType, Clone, Copy)]
pub struct TilemapUniform {
    pub translation: Vec2,
    pub tile_render_size: Vec2,
    pub tile_render_scale: Vec2,
    pub tile_slot_size: Vec2,
    pub anchor: Vec2,
    pub texture_size: Vec2,
}

#[derive(Resource, Default)]
pub struct TilemapUniformsStorage {
    buffer: DynamicUniformBuffer<TilemapUniform>,
}

impl TilemapUniformsStorage {
    /// Update the uniform buffer with the current tilemap uniforms.
    /// Returns the `TilemapUniform` component to be used in the tilemap render pass.
    pub fn insert(
        &mut self,
        tilemap: &ExtractedTilemap,
    ) -> DynamicUniformComponent<TilemapUniform> {
        let (texture_size, tile_render_size) = if let Some(texture) = &tilemap.texture {
            let desc = texture.desc();
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
            (Vec2::ZERO, Vec2::ZERO)
        };

        let component = TilemapUniform {
            translation: tilemap.translation,
            tile_render_size,
            tile_slot_size: tilemap.tile_slot_size,
            tile_render_scale: tilemap.tile_render_scale,
            anchor: tilemap.anchor,
            texture_size,
        };

        let index = self.buffer.push(component);

        DynamicUniformComponent { index, component }
    }

    /// Get the binding resource for the uniform buffer.
    pub fn binding(&self) -> Option<BindingResource> {
        self.buffer.binding()
    }

    /// Clear the uniform buffer.
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Write the uniform buffer to the GPU.
    pub fn write(&mut self, render_device: &RenderDevice, render_queue: &RenderQueue) {
        self.buffer.write_buffer(render_device, render_queue);
    }
}
