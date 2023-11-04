use bevy::{
    prelude::{Component, Mat4, Resource, Vec2},
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
    pub transform: Mat4,
    pub tile_size: Vec2,
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
        let component = TilemapUniform {
            transform: tilemap.transform_matrix,
            tile_size: tilemap.tile_render_size,
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
