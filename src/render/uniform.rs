use bevy::{
    prelude::{Component, Mat4, Resource, UVec2},
    render::{
        render_resource::{DynamicUniformBuffer, ShaderType},
        renderer::{RenderDevice, RenderQueue},
    },
};

use super::extract::ExtractedTilemap;

#[derive(ShaderType, Component, Clone, Copy)]
pub struct TilemapUniform {
    pub index: u32,
    pub transform: Mat4,
    pub tile_size: UVec2,
}

#[derive(Resource, Default)]
pub struct TilemapUniformsStorage {
    pub buffer_size: u32,
    pub buffer: DynamicUniformBuffer<TilemapUniform>,
}

impl TilemapUniformsStorage {
    /// Update the uniform buffer with the current tilemap uniforms.
    /// Returns the `TilemapUniform` component to be used in the tilemap render pass.
    pub fn insert(&mut self, tilemap: &ExtractedTilemap) -> TilemapUniform {
        let component = TilemapUniform {
            index: self.buffer_size,
            transform: tilemap.transform,
            tile_size: tilemap.tile_size,
        };

        self.buffer.push(component);
        self.buffer_size += 1;

        component
    }

    /// Clear the uniform buffer.
    pub fn reset(&mut self) {
        self.buffer.clear();
    }

    /// Write the uniform buffer to the GPU.
    pub fn write(&mut self, render_device: &RenderDevice, render_queue: &RenderQueue) {
        self.buffer.write_buffer(render_device, render_queue);
    }
}
