use bevy::{
    ecs::component::Component,
    prelude::{Assets, Commands, Entity, Handle, IVec2, Image, ResMut, UVec2, Vec2},
    render::render_resource::TextureUsages,
};

use crate::{
    math::{aabb::AabbBox2d, FillArea},
    render::texture::TilemapTextureDescriptor,
};

use super::tile::{TileBuilder, TileFlip, TileType, TilemapTexture};

pub struct TilemapBuilder {
    pub(crate) tile_type: TileType,
    pub(crate) size: UVec2,
    pub(crate) tile_render_scale: Vec2,
    pub(crate) tile_slot_size: Vec2,
    pub(crate) anchor: Vec2,
    pub(crate) render_chunk_size: u32,
    pub(crate) texture: Option<TilemapTexture>,
    pub(crate) translation: Vec2,
    pub(crate) z_order: f32,
    pub(crate) flip: u32,
}

impl TilemapBuilder {
    /// Create a new tilemap builder.
    pub fn new(ty: TileType, size: UVec2, tile_slot_size: Vec2) -> Self {
        Self {
            tile_type: ty,
            size,
            tile_render_scale: Vec2::ONE,
            tile_slot_size,
            anchor: Vec2 { x: 0.5, y: 0. },
            texture: None,
            render_chunk_size: 32,
            translation: Vec2::ZERO,
            z_order: 0.,
            flip: 0,
        }
    }

    /// Override z order. Default is 10.
    /// The higher the value of z_order, the less likely to be covered.
    pub fn with_z_order(&mut self, z_order: f32) -> &mut Self {
        self.z_order = z_order;
        self
    }

    /// Override render chunk size. Default is 32.
    pub fn with_render_chunk_size(&mut self, size: u32) -> &mut Self {
        self.render_chunk_size = size;
        self
    }

    /// Override translation. Default is `Vec2::ZERO`.
    pub fn with_translation(&mut self, translation: Vec2) -> &mut Self {
        self.translation = translation;
        self
    }

    /// Assign a texture to the tilemap.
    pub fn with_texture(
        &mut self,
        texture: Handle<Image>,
        desc: TilemapTextureDescriptor,
    ) -> &mut Self {
        self.texture = Some(TilemapTexture { texture, desc });
        self
    }

    /// Flip the uv of tiles. Use `TileFlip::Horizontal | TileFlip::Vertical` to flip both.
    pub fn with_flip(&mut self, flip: TileFlip) -> &mut Self {
        self.flip |= flip as u32;
        self
    }

    /// Override the anchor of the tile. Default is `[0.5, 0.]`.
    ///
    /// This can be useful when rendering `non_uniform` tilemaps. ( See the example )
    pub fn with_anchor(&mut self, anchor: Vec2) -> &mut Self {
        assert!(
            anchor.x >= 0. && anchor.x <= 1. && anchor.y >= 0. && anchor.y <= 1.,
            "Anchor must be in range [0, 1]"
        );
        self.anchor = anchor;
        self
    }

    /// Override the tile render scale. Default is `Vec2::ONE` which means the render size is equal to the pixel size.
    pub fn with_tile_render_scale(&mut self, tile_render_scale: Vec2) -> &mut Self {
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
            tile_slot_size: self.tile_slot_size,
            anchor: self.anchor,
            texture: self.texture.clone(),
            flip: self.flip,
            aabb: AabbBox2d::from_tilemap_builder(self),
            translation: self.translation,
            z_order: self.z_order,
        };
        entity.insert((WaitForTextureUsageChange, tilemap.clone()));
        (entity.id(), tilemap)
    }
}

#[cfg(feature = "serializing")]
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct TilemapPattern {
    pub size: UVec2,
    pub tiles: Vec<Option<crate::serializing::SerializedTile>>,
}

impl TilemapPattern {
    pub fn get(&self, index: UVec2) -> Option<&crate::serializing::SerializedTile> {
        self.tiles[(index.y * self.size.x + index.x) as usize].as_ref()
    }
}

#[derive(Component, Clone)]
pub struct Tilemap {
    pub(crate) id: Entity,
    pub(crate) tile_type: TileType,
    pub(crate) size: UVec2,
    pub(crate) tile_render_scale: Vec2,
    pub(crate) tile_slot_size: Vec2,
    pub(crate) anchor: Vec2,
    pub(crate) render_chunk_size: u32,
    pub(crate) texture: Option<TilemapTexture>,
    pub(crate) tiles: Vec<Option<Entity>>,
    pub(crate) flip: u32,
    pub(crate) aabb: AabbBox2d,
    pub(crate) translation: Vec2,
    pub(crate) z_order: f32,
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
        self.tiles[self.grid_index_to_linear_index(index)]
    }

    /// Set a tile.
    ///
    /// Overwrites the tile if it already exists.
    pub fn set(&mut self, commands: &mut Commands, index: UVec2, tile_builder: &TileBuilder) {
        if self.is_out_of_tilemap_uvec(index) {
            return;
        }

        self.set_unchecked(commands, index, tile_builder);
    }

    pub(crate) fn set_unchecked(
        &mut self,
        commands: &mut Commands,
        index: UVec2,
        tile_builder: &TileBuilder,
    ) {
        let linear_index = self.grid_index_to_linear_index(index);
        if let Some(previous) = self.tiles[linear_index] {
            commands.entity(previous).despawn();
        }
        let new_tile = tile_builder.build(commands, index, self);
        self.tiles[linear_index] = Some(new_tile);
    }

    /// Remove a tile.
    pub fn remove(&mut self, commands: &mut Commands, index: UVec2) {
        if self.is_out_of_tilemap_uvec(index) || self.get(index).is_none() {
            return;
        }

        self.remove_unchecked(commands, index);
    }

    pub(crate) fn remove_unchecked(&mut self, commands: &mut Commands, index: UVec2) {
        let index = self.grid_index_to_linear_index(index);
        commands.entity(self.tiles[index].unwrap()).despawn();
        self.tiles[index] = None;
    }

    /// Fill a rectangle area with tiles.
    pub fn fill_rect(
        &mut self,
        commands: &mut Commands,
        area: FillArea,
        tile_builder: &TileBuilder,
    ) {
        let builder = tile_builder.clone();
        for y in area.origin.y..=area.dest.y {
            for x in area.origin.x..=area.dest.x {
                self.set_unchecked(commands, UVec2 { x, y }, &builder);
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
                let builder = tile_builder(if relative_index {
                    UVec2::new(x, y) - area.origin
                } else {
                    UVec2::new(x, y)
                });
                self.set_unchecked(commands, UVec2 { x, y }, &builder);
            }
        }
    }

    /// This is a method for debugging.
    /// You can check if the uvs of tiles are correct.
    ///
    /// Fill the tilemap with tiles from the atlas in a row.
    /// Notice that the method **won't** check if the tilemap is wide enough.
    #[cfg(feature = "debug")]
    pub fn fill_atlas(&mut self, commands: &mut Commands) {
        if let Some(texture) = &self.texture {
            for x in 0..texture.desc.tiles_uv.len() as u32 {
                self.set_unchecked(commands, UVec2 { x, y: 0 }, &TileBuilder::new(x));
            }
        }
    }

    #[cfg(feature = "serializing")]
    pub fn set_pattern(&mut self, commands: &mut Commands, pattern: TilemapPattern, origin: UVec2) {
        for y in 0..pattern.size.y {
            for x in 0..pattern.size.x {
                let index = UVec2 { x, y };
                if let Some(tile) = pattern.get(index) {
                    self.set(
                        commands,
                        index + origin,
                        &TileBuilder::from_serialized_tile(tile),
                    );
                }
            }
        }
    }

    #[inline]
    /// Get the id of the tilemap.
    pub fn id(&self) -> Entity {
        self.id
    }

    /// Get the world position of the center of a slot.
    pub fn index_to_world(&self, index: UVec2) -> Vec2 {
        let index = index.as_vec2();
        match self.tile_type {
            TileType::Square => (index - self.anchor) * self.tile_slot_size + self.translation,
            TileType::IsometricDiamond => {
                (Vec2 {
                    x: (index.x - index.y - 1.),
                    y: (index.x + index.y),
                } / 2.
                    - self.anchor)
                    * self.tile_slot_size
                    + self.translation
            }
        }
    }

    #[inline]
    pub fn grid_index_to_linear_index(&self, index: UVec2) -> usize {
        (index.y * self.size.x + index.x) as usize
    }

    #[inline]
    pub fn is_out_of_tilemap_uvec(&self, index: UVec2) -> bool {
        index.x >= self.size.x || index.y >= self.size.y
    }

    #[inline]
    pub fn is_out_of_tilemap_ivec(&self, index: IVec2) -> bool {
        index.x < 0 || index.y < 0 || index.x >= self.size.x as i32 || index.y >= self.size.y as i32
    }

    #[inline]
    pub fn get_tile_convex_hull(&self, index: UVec2) -> [Vec2; 4] {
        let offset = self.index_to_world(index);
        let (x, y) = (self.tile_slot_size.x, self.tile_slot_size.y);
        match self.tile_type {
            TileType::Square => [
                (Vec2 { x: 0., y: 0. } + offset).into(),
                (Vec2 { x: 0., y } + offset).into(),
                (Vec2 { x, y } + offset).into(),
                (Vec2 { x, y: 0. } + offset).into(),
            ],
            TileType::IsometricDiamond => [
                (Vec2 { x: 0., y: y / 2. } + offset).into(),
                (Vec2 { x: x / 2., y } + offset).into(),
                (Vec2 { x, y: y / 2. } + offset).into(),
                (Vec2 { x: x / 2., y: 0. } + offset).into(),
            ],
        }
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
