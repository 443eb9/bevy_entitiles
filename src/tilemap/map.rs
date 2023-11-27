use bevy::{
    ecs::component::Component,
    prelude::{Assets, Commands, Entity, Handle, IVec2, Image, ResMut, UVec2, Vec2},
    render::render_resource::{FilterMode, TextureUsages},
};

use crate::{
    math::{aabb::AabbBox2d, FillArea},
    render::{chunk::RenderChunkStorage, texture::TilemapTextureDescriptor},
};

use super::tile::{TileBuilder, TileFlip, TileType, TilemapTexture};

pub struct TilemapBuilder {
    pub(crate) tile_type: TileType,
    pub(crate) size: UVec2,
    pub(crate) tile_render_scale: Vec2,
    pub(crate) tile_grid_size: Vec2,
    pub(crate) anchor: Vec2,
    pub(crate) render_chunk_size: u32,
    pub(crate) texture: Option<TilemapTexture>,
    pub(crate) filter_mode: FilterMode,
    pub(crate) translation: Vec2,
    pub(crate) z_order: u32,
    pub(crate) flip: u32,
    pub(crate) is_uniform: bool,
}

impl TilemapBuilder {
    /// Create a new tilemap builder.
    pub fn new(ty: TileType, size: UVec2, tile_grid_size: Vec2) -> Self {
        Self {
            tile_type: ty,
            size,
            tile_render_scale: Vec2::ONE,
            tile_grid_size,
            anchor: Vec2::ZERO,
            texture: None,
            render_chunk_size: 32,
            filter_mode: FilterMode::Nearest,
            translation: Vec2::ZERO,
            z_order: 10,
            flip: 0,
            is_uniform: true,
        }
    }

    /// Override z order. Default is 10.
    /// The higher the value of z_order, the less likely to be covered.
    pub fn with_z_order(mut self, z_order: u32) -> Self {
        self.z_order = z_order;
        self
    }

    /// Override render chunk size. Default is 32.
    pub fn with_render_chunk_size(mut self, size: u32) -> Self {
        self.render_chunk_size = size;
        self
    }

    /// Override filter mode. Default is nearest.
    pub fn with_filter_mode(mut self, filter_mode: FilterMode) -> Self {
        self.filter_mode = filter_mode;
        self
    }

    /// Override translation. Default is `Vec2::ZERO`.
    pub fn with_translation(mut self, translation: Vec2) -> Self {
        self.translation = translation;
        self
    }

    pub fn with_uniform_texture(
        mut self,
        texture: Handle<Image>,
        count: UVec2,
        filter_mode: FilterMode,
    ) -> Self {
        self.texture = Some(TilemapTexture {
            texture,
            desc: TilemapTextureDescriptor::from_full_grid(count, filter_mode),
        });
        self.is_uniform = true;
        self
    }

    /// Assign a texture to the tilemap.
    pub fn with_non_uniform_texture(
        mut self,
        texture_size: UVec2,
        texture: Handle<Image>,
        desc: TilemapTextureDescriptor,
    ) -> Self {
        self.texture = Some(TilemapTexture { texture, desc });
        self
    }

    /// Flip the uv of tiles. Use `TileFlip::Horizontal | TileFlip::Vertical` to flip both.
    pub fn with_flip(mut self, flip: TileFlip) -> Self {
        self.flip |= flip as u32;
        self
    }

    /// Override the anchor of the tile. Default is `Vec2::ZERO`.
    ///
    /// This can be useful when rendering `non_uniform` tilemaps. ( See the example )
    pub fn with_anchor(mut self, anchor: Vec2) -> Self {
        assert!(
            anchor.x >= 0. && anchor.x <= 1. && anchor.y >= 0. && anchor.y <= 1.,
            "Anchor must be in range [0, 1]"
        );
        self.anchor = anchor;
        self
    }

    /// Override the tile render scale. Default is `Vec2::ONE`.
    pub fn with_tile_render_scale(mut self, tile_render_scale: Vec2) -> Self {
        self.tile_render_scale = tile_render_scale;
        self
    }

    /// Build the tilemap and spawn it into the world.
    /// You can modify the component and insert it back.
    pub fn build(&self, commands: &mut Commands) -> (Entity, Tilemap) {
        let mut entity = commands.spawn_empty();
        let tilemap = Tilemap {
            tiles: vec![None; (self.size.x * self.size.y) as usize],
            id: entity.id(),
            tile_render_scale: self.tile_render_scale,
            tile_type: self.tile_type,
            size: self.size,
            render_chunk_size: self.render_chunk_size,
            tile_grid_size: self.tile_grid_size,
            anchor: self.anchor,
            texture: self.texture.clone(),
            filter_mode: self.filter_mode,
            flip: self.flip,
            aabb: AabbBox2d::from_tilemap_builder(self),
            translation: self.translation,
            z_order: self.z_order,
        };
        entity.insert((WaitForTextureUsageChange, tilemap.clone()));
        (entity.id(), tilemap)
    }
}

#[derive(Component, Clone)]
pub struct Tilemap {
    pub(crate) id: Entity,
    pub(crate) tile_type: TileType,
    pub(crate) size: UVec2,
    pub(crate) tile_render_scale: Vec2,
    pub(crate) tile_grid_size: Vec2,
    pub(crate) anchor: Vec2,
    pub(crate) render_chunk_size: u32,
    pub(crate) texture: Option<TilemapTexture>,
    pub(crate) filter_mode: FilterMode,
    pub(crate) tiles: Vec<Option<Entity>>,
    pub(crate) flip: u32,
    pub(crate) aabb: AabbBox2d,
    pub(crate) translation: Vec2,
    pub(crate) z_order: u32,
}

impl Tilemap {
    /// Get a tile.
    pub fn get(&self, index: UVec2) -> Option<Entity> {
        if self.is_out_of_tilemap_uvec(index) {
            return None;
        }

        self.get_unchecked(index)
    }

    pub(crate) fn get_unchecked(&self, index: UVec2) -> Option<Entity> {
        self.tiles[(index.y * self.size.x + index.x) as usize]
    }

    /// Set a tile.
    ///
    /// Overwrites the tile if it already exists.
    pub fn set(&mut self, commands: &mut Commands, tile_builder: &TileBuilder) {
        if self.is_out_of_tilemap_uvec(tile_builder.index) {
            return;
        }

        self.set_unchecked(commands, tile_builder);
    }

    pub(crate) fn set_unchecked(&mut self, commands: &mut Commands, tile_builder: &TileBuilder) {
        let index = (tile_builder.index.y * self.size.x + tile_builder.index.x) as usize;
        if let Some(previous) = self.tiles[index] {
            commands.entity(previous).despawn();
        }
        let new_tile = tile_builder.build(commands, self);
        self.tiles[index] = Some(new_tile);
    }

    /// Remove a tile.
    pub fn remove(&mut self, commands: &mut Commands, index: UVec2) {
        if self.is_out_of_tilemap_uvec(index) || self.get(index).is_none() {
            return;
        }

        self.remove_unchecked(commands, index);
    }

    pub(crate) fn remove_unchecked(&mut self, commands: &mut Commands, index: UVec2) {
        let index = (index.y * self.size.x + index.x) as usize;
        commands.entity(self.tiles[index].unwrap()).despawn();
        self.tiles[index] = None;
    }

    /// Fill a rectangle area with tiles.
    ///
    /// You don't need to assign the `index` in the `tile_builder`.
    pub fn fill_rect(
        &mut self,
        commands: &mut Commands,
        area: FillArea,
        tile_builder: &TileBuilder,
    ) {
        let mut builder = tile_builder.clone();
        for y in area.origin.y..=area.dest.y {
            for x in area.origin.x..=area.dest.x {
                builder.index = UVec2::new(x, y);
                self.set_unchecked(commands, &builder);
            }
        }
    }

    /// Fill a rectangle area with tiles returned by `tile_builder`.
    ///
    /// Set `relative_index` to true if your function takes index relative to the area origin.
    pub fn fill_rect_custom(
        &mut self,
        commands: &mut Commands,
        area: FillArea,
        mut tile_builder: impl FnMut(UVec2) -> TileBuilder,
        relative_index: bool,
    ) {
        for y in area.origin.y..=area.dest.y {
            for x in area.origin.x..=area.dest.x {
                let mut builder = tile_builder(if relative_index {
                    UVec2::new(x, y) - area.origin
                } else {
                    UVec2::new(x, y)
                });
                builder.index = UVec2::new(x, y);
                self.set_unchecked(commands, &builder);
            }
        }
    }

    /// Get the id of the tilemap.
    pub fn get_id(&self) -> Entity {
        self.id
    }

    /// Get the world position of the center of a tile.
    pub fn index_to_world(&self, index: UVec2) -> Vec2 {
        let index = index.as_vec2();
        match self.tile_type {
            TileType::Square => Vec2 {
                x: (index.x + 0.5) * self.tile_render_scale.x + self.translation.x,
                y: (index.y + 0.5) * self.tile_render_scale.y + self.translation.y,
            },
            TileType::IsometricDiamond => Vec2 {
                x: (index.x - index.y) / 2. * self.tile_render_scale.x + self.translation.x,
                y: (index.x + index.y + 1.) / 2. * self.tile_render_scale.y + self.translation.y,
            },
        }
    }

    pub fn is_out_of_tilemap_uvec(&self, index: UVec2) -> bool {
        index.x >= self.size.x || index.y >= self.size.y
    }

    pub fn is_out_of_tilemap_ivec(&self, index: IVec2) -> bool {
        index.x < 0 || index.y < 0 || index.x >= self.size.x as i32 || index.y >= self.size.y as i32
    }

    /// Bevy doesn't set the `COPY_SRC` usage for images by default, so we need to do it manually.
    pub(crate) fn set_usage(
        &mut self,
        commands: &mut Commands,
        image_assets: &mut ResMut<Assets<Image>>,
    ) {
        let Some(texture) = &self.texture else {
            commands
                .entity(self.id)
                .remove::<WaitForTextureUsageChange>();
            return;
        };

        let Some(image) = image_assets.get(&texture.clone_weak()) else {
            return;
        };

        if !image
            .texture_descriptor
            .usage
            .contains(TextureUsages::COPY_SRC)
        {
            image_assets
                .get_mut(&texture.clone_weak())
                .unwrap()
                .texture_descriptor
                .usage
                .set(TextureUsages::COPY_SRC, true);
        }

        commands
            .entity(self.id)
            .remove::<WaitForTextureUsageChange>();
    }
}

#[derive(Component)]
pub struct WaitForTextureUsageChange;
