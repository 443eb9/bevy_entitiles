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
    pub transform: Vec2,
    pub tile_render_scale: Vec2,
    pub tile_grid_size: Vec2,
    pub anchor: Vec2,
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
            transform: tilemap.translation,
            tile_grid_size: tilemap.tile_grid_size,
            tile_render_scale: tilemap.tile_render_scale,
            anchor: tilemap.anchor,
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
