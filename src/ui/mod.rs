use bevy::{
    app::{App, Plugin, Update},
    asset::{load_internal_asset, Asset, Assets, Handle},
    ecs::system::{Res, ResMut, Resource},
    math::{UVec2, Vec2, Vec4},
    reflect::TypePath,
    render::{
        render_resource::{AsBindGroup, Shader, ShaderRef, ShaderType},
        texture::Image,
    },
    time::Time,
    ui::{UiMaterial, UiMaterialPlugin},
    utils::HashMap,
};

use crate::{render::buffer::TileAnimation, tilemap::tile::TileFlip};

pub const UI_TILES_SHADER: Handle<Shader> = Handle::weak_from_u128(213513554364645316312);

pub struct EntiTilesUiPlugin;

impl Plugin for EntiTilesUiPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(app, UI_TILES_SHADER, "ui_tiles.wgsl", Shader::from_wgsl);

        app.add_plugins(UiMaterialPlugin::<UiTileMaterial>::default());
        app.init_resource::<UiTileMaterialRegistry>();
        app.add_systems(Update, materials_time_updater);
    }
}

pub struct UiTilemapTexture {
    pub(crate) texture: Handle<Image>,
    pub(crate) desc: UiTilemapTextureDescriptor,
}

impl UiTilemapTexture {
    pub fn new(texture: Handle<Image>, desc: UiTilemapTextureDescriptor) -> Self {
        Self { texture, desc }
    }

    #[inline]
    pub fn handle(&self) -> &Handle<Image> {
        &self.texture
    }
}

pub struct UiTilemapTextureDescriptor {
    pub(crate) size: UVec2,
    pub(crate) tile_size: UVec2,
}

impl UiTilemapTextureDescriptor {
    pub fn new(size: UVec2, tile_size: UVec2) -> Self {
        Self { size, tile_size }
    }
}

#[derive(Resource, Default)]
pub struct UiTileMaterialRegistry {
    materials: HashMap<Handle<Image>, Vec<Handle<UiTileMaterial>>>,
}

impl UiTileMaterialRegistry {
    pub fn register(
        &mut self,
        assets: &mut ResMut<Assets<UiTileMaterial>>,
        texture: &UiTilemapTexture,
        builder: &UiTileBuilder,
    ) {
        self.materials
            .entry(texture.texture.clone_weak())
            .or_default()
            .push(assets.add(UiTileMaterial {
                texture: texture.texture.clone(),
                uniform: builder.build(&texture.desc).into(),
            }));
    }

    pub fn register_many(
        &mut self,
        assets: &mut ResMut<Assets<UiTileMaterial>>,
        texture: &UiTilemapTexture,
        builders: Vec<UiTileBuilder>,
    ) {
        let count = texture.desc.size / texture.desc.tile_size;

        for y in 0..count.y {
            for x in 0..count.x {
                let builder = &builders[(y * count.x + x) as usize];
                self.register(assets, texture, builder);
            }
        }
    }

    pub fn get_handle(
        &self,
        texture: &Handle<Image>,
        texture_index: usize,
    ) -> Option<Handle<UiTileMaterial>> {
        self.materials
            .get(texture)
            .and_then(|mats| mats.get(texture_index).cloned())
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &Handle<UiTileMaterial>> {
        self.materials.iter().map(|(_, mats)| mats.iter()).flatten()
    }
}

pub struct UiTileBuilder {
    pub(crate) color: Vec4,
    pub(crate) flip: u32,
    pub(crate) texture_index: u32,
    pub(crate) animation: Option<TileAnimation>,
}

impl UiTileBuilder {
    pub fn new() -> Self {
        Self {
            color: Vec4::ONE,
            flip: 0,
            texture_index: 0,
            animation: None,
        }
    }

    pub fn with_color(mut self, color: Vec4) -> Self {
        self.color = color;
        self
    }

    pub fn with_flip(mut self, flip: TileFlip) -> Self {
        self.flip |= flip as u32;
        self
    }

    pub fn with_animation(mut self, animation: TileAnimation) -> Self {
        self.animation = Some(animation);
        self
    }

    pub fn with_texture_index(mut self, texture_index: u32) -> Self {
        self.texture_index = texture_index;
        self
    }

    pub fn build(&self, desc: &UiTilemapTextureDescriptor) -> UiTileUniform {
        UiTileUniform {
            color: self.color,
            atlas_size: desc.tile_size.as_vec2(),
            atlas_count: desc.size / desc.tile_size,
            texture_index: self.texture_index,
            flip: self.flip,
            time: 0.,
            anim: self.animation.unwrap_or_default(),
        }
    }

    pub fn fill_grid_with_atlas(&self, desc: &UiTilemapTextureDescriptor) -> Vec<Self> {
        let count = desc.size / desc.tile_size;
        let mut builders = Vec::with_capacity(count.x as usize * count.y as usize);
        for y in 0..count.y {
            for x in 0..count.x {
                builders.push(Self {
                    color: self.color,
                    flip: self.flip,
                    texture_index: y * count.x + x,
                    animation: self.animation.clone(),
                });
            }
        }
        builders
    }
}

#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub struct UiTileMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub texture: Handle<Image>,
    #[uniform(2)]
    pub uniform: UiTileUniform,
}

impl UiMaterial for UiTileMaterial {
    fn fragment_shader() -> ShaderRef {
        UI_TILES_SHADER.into()
    }
}

#[derive(ShaderType, Debug, Clone)]
pub struct UiTileUniform {
    pub color: Vec4,
    pub atlas_size: Vec2,
    pub atlas_count: UVec2,
    pub texture_index: u32,
    pub flip: u32,
    pub time: f32,
    pub anim: TileAnimation,
}

pub fn materials_time_updater(
    mut asstes: ResMut<Assets<UiTileMaterial>>,
    mats_reg: Res<UiTileMaterialRegistry>,
    time: Res<Time>,
) {
    mats_reg.iter().for_each(|handle| {
        asstes.get_mut(handle).unwrap().uniform.time = time.elapsed_seconds();
    });
}
