use bevy::{
    app::Plugin,
    asset::{load_internal_asset, Asset, Assets, Handle},
    ecs::system::ResMut,
    math::{Vec2, Vec4},
    reflect::TypePath,
    render::{
        render_resource::{AsBindGroup, Shader},
        texture::Image,
    },
    ui::{UiMaterial, UiMaterialPlugin},
};

use crate::render::texture::TilemapTextureDescriptor;

pub const UI_TILES_SHADER: Handle<Shader> = Handle::weak_from_u128(213513554364645316312);

pub struct EntiTilesUiPlugin;

impl Plugin for EntiTilesUiPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        load_internal_asset!(app, UI_TILES_SHADER, "ui_tiles.wgsl", Shader::from_wgsl);

        app.add_plugins(UiMaterialPlugin::<UiTileMaterial>::default());
    }
}

pub struct UiTilemapTexture {
    pub(crate) texture: Handle<Image>,
    pub(crate) desc: TilemapTextureDescriptor,
}

impl UiTilemapTexture {
    /// Add all the textures to `Assets<UiMaterial>`
    pub fn register_materials(
        self,
        color: Option<Vec4>,
        assets: &mut ResMut<Assets<UiTileMaterial>>,
    ) -> UiTileMaterialsLookup {
        let count = self.desc.size / self.desc.tile_size;
        let mut handles = Vec::with_capacity((count.x * count.y) as usize);
        let color = color.unwrap_or(Vec4::ONE);

        for y in 0..count.y {
            for x in 0..count.x {
                let min = Vec2::new(x as f32, y as f32) * self.desc.tile_size.as_vec2();
                let max = min + self.desc.tile_size.as_vec2();
                handles.push(assets.add(UiTileMaterial {
                    texture: self.texture.clone(),
                    uv: Vec4::new(min.x, min.y, max.x, max.y),
                    color,
                    texture_size: self.desc.size.as_vec2(),
                }));
            }
        }
        UiTileMaterialsLookup { materials: handles }
    }
}

#[derive(Default)]
pub struct UiTileMaterialsLookup {
    pub(crate) materials: Vec<Handle<UiTileMaterial>>,
}

impl UiTileMaterialsLookup {
    pub fn clone(&self, index: u32) -> Option<Handle<UiTileMaterial>> {
        if let Some(h) = self.materials.get(index as usize) {
            Some(h.clone())
        } else {
            None
        }
    }
}

#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub struct UiTileMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub texture: Handle<Image>,
    #[uniform(2)]
    pub uv: Vec4,
    #[uniform(2)]
    pub color: Vec4,
    #[uniform(2)]
    pub texture_size: Vec2,
}

impl UiMaterial for UiTileMaterial {
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        UI_TILES_SHADER.into()
    }
}
