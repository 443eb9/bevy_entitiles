use bevy::{
    ecs::system::Resource,
    math::Vec2,
    render::render_resource::{DynamicUniformBuffer, ShaderType},
};

use crate::render::uniform::{DynamicUniformComponent, UniformsStorage};

use super::render::ExtractedUiTilemap;

#[derive(ShaderType, Clone, Copy)]
pub struct UiTilemapUniform {
    pub anchor: Vec2,
    pub texture_size: Vec2,
}

#[derive(Resource, Default)]
pub struct UiTilemapUniformsStorage {
    buffer: DynamicUniformBuffer<UiTilemapUniform>,
}

impl UniformsStorage<ExtractedUiTilemap, UiTilemapUniform> for UiTilemapUniformsStorage {
    fn insert(
        &mut self,
        extracted: &ExtractedUiTilemap,
    ) -> DynamicUniformComponent<UiTilemapUniform> {
        let component = UiTilemapUniform {
            anchor: extracted.anchor,
            texture_size: extracted.texture.desc.size.as_vec2(),
        };
        DynamicUniformComponent {
            index: self.buffer.push(component),
            component,
        }
    }

    #[inline]
    fn buffer(&mut self) -> &mut DynamicUniformBuffer<UiTilemapUniform> {
        &mut self.buffer
    }
}
