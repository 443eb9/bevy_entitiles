use bevy::{
    prelude::{
        Assets, Commands, Component, Entity, Handle, IVec2, Image, ResMut, UVec2, Vec2, Vec4,
    },
    render::{
        mesh::MeshVertexAttribute,
        render_resource::{FilterMode, TextureUsages, VertexFormat},
    },
};

use crate::{
    math::{aabb::AabbBox2d, FillArea},
    render::{chunk::RenderChunkStorage, texture::TilemapTextureDescriptor},
};

#[cfg(feature = "algorithm")]
pub mod algo_tilemap;
#[cfg(any(feature = "physics_xpbd", feature = "physics_rapier"))]
pub mod physics;

pub const TILEMAP_MESH_ATTR_INDEX: MeshVertexAttribute =
    MeshVertexAttribute::new("GridIndex", 14513156146, VertexFormat::Float32x2);
pub const TILEMAP_MESH_ATTR_TEXTURE_INDEX: MeshVertexAttribute =
    MeshVertexAttribute::new("TextureIndex", 186541653135, VertexFormat::Uint32);
pub const TILEMAP_MESH_ATTR_COLOR: MeshVertexAttribute =
    MeshVertexAttribute::new("Color", 85415341854, VertexFormat::Float32x4);

#[derive(Default, PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum TileType {
    /// The most basic shape.
    #[default]
    Square,
    /// A diamond shape. It's like a square but rotated 45 degrees counterclockwise around the origin.
    /// But the coordinate system is the same as `Square`.
    IsometricDiamond,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum TileFlip {
    Horizontal = 1u32 << 0,
    Vertical = 1u32 << 1,
}

#[derive(Component, Clone)]
pub struct TileAnimation {
    pub sequence: Vec<u32>,
    pub fps: f32,
    pub is_loop: bool,
}

#[derive(Clone)]
pub struct TileTexture {
    texture: Handle<Image>,
    desc: TilemapTextureDescriptor,
}

impl TileTexture {
    pub fn clone_weak(&self) -> Handle<Image> {
        self.texture.clone_weak()
    }

    pub fn get_desc(&self) -> &TilemapTextureDescriptor {
        &self.desc
    }

    pub fn get_handle(&self) -> &Handle<Image> {
        &self.texture
    }
}

#[derive(Clone)]
pub struct TileBuilder {
    index: UVec2,
    texture_index: u32,
    anim: Option<TileAnimation>,
    color: Vec4,
}

impl TileBuilder {
    /// Create a new tile builder.
    pub fn new(index: UVec2, texture_index: u32) -> Self {
        Self {
            index,
            texture_index,
            anim: None,
            color: Vec4::ONE,
        }
    }

    pub fn with_color(&mut self, color: Vec4) -> &mut Self {
        self.color = color;
        self
    }

    pub fn with_animation(&mut self, anim: TileAnimation) -> &mut Self {
        self.anim = Some(anim);
        self
    }

    /// Build the tile and spawn it.
    ///
    /// # Note
    /// DO NOT call this method manually unless you really need to.
    ///
    /// Use `Tilemap::set` or `Tilemap::fill_xxx` instead.
    pub fn build(&self, commands: &mut Commands, tilemap: &Tilemap) -> Entity {
        let render_chunk_index_2d = self.index / tilemap.render_chunk_size;
        let render_chunk_index = {
            if tilemap.size.x % tilemap.render_chunk_size == 0 {
                render_chunk_index_2d.y * (tilemap.size.x / tilemap.render_chunk_size)
                    + render_chunk_index_2d.x
            } else {
                render_chunk_index_2d.y * (tilemap.size.x / tilemap.render_chunk_size + 1)
                    + render_chunk_index_2d.x
            }
        } as usize;
        let mut tile = commands.spawn_empty();
        tile.insert(Tile {
            render_chunk_index,
            tilemap_id: tilemap.id,
            index: self.index,
            texture_index: self.texture_index,
            color: self.color,
        });
        if let Some(anim) = &self.anim {
            tile.insert(anim.clone());
        }
        tile.id()
    }
}

#[derive(Component, Clone, Debug)]
pub struct Tile {
    pub tilemap_id: Entity,
    pub render_chunk_index: usize,
    pub index: UVec2,
    pub texture_index: u32,
    pub color: Vec4,
}

pub struct TilemapBuilder {
    pub(crate) tile_type: TileType,
    pub(crate) size: UVec2,
    pub(crate) tile_size: UVec2,
    pub(crate) tile_render_size: Vec2,
    pub(crate) render_chunk_size: u32,
    pub(crate) texture: Option<TileTexture>,
    pub(crate) filter_mode: FilterMode,
    pub(crate) translation: Vec2,
    pub(crate) z_order: u32,
    pub(crate) flip: u32,
    pub(crate) safety_check: bool,
}

impl TilemapBuilder {
    /// Create a new tilemap builder.
    ///
    /// # Notice
    /// The id should **must** be unique. Randomly generated ids are recommended.
    ///
    /// `tile_size` is the size of the tile in pixels while `tile_render_size` is the size of the tile in the world.
    pub fn new(ty: TileType, size: UVec2, tile_render_size: Vec2) -> Self {
        Self {
            tile_type: ty,
            size,
            tile_size: UVec2::ZERO,
            tile_render_size,
            texture: None,
            render_chunk_size: 32,
            filter_mode: FilterMode::Nearest,
            translation: Vec2::ZERO,
            z_order: 10,
            flip: 0,
            safety_check: true,
        }
    }

    /// Override z order. Default is 10.
    /// The higher the value of z_order, the earlier it is rendered.
    pub fn with_z_order(&mut self, z_order: u32) -> &mut Self {
        self.z_order = z_order;
        self
    }

    /// Override render chunk size. Default is 32.
    pub fn with_render_chunk_size(&mut self, size: u32) -> &mut Self {
        self.render_chunk_size = size;
        self
    }

    /// Override filter mode. Default is nearest.
    pub fn with_filter_mode(&mut self, filter_mode: FilterMode) -> &mut Self {
        self.filter_mode = filter_mode;
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
        self.texture = Some(TileTexture { texture, desc });
        self.tile_size = desc.tile_size;
        self
    }

    // /// Align the whole rendered tilemap to the center.
    // ///
    // /// It will override the `transform` you set.
    // pub fn with_center(&mut self, center: Vec2) -> &mut Self {
    //     self.translation = Vec2::new(
    //         center.x - self.tile_render_size.x / 2. * self.size.x as f32,
    //         center.y - self.tile_render_size.y / 2. * self.size.y as f32,
    //     );
    //     self
    // }

    /// Flip the uv of tiles. Use `TileFlip::Horizontal | TileFlip::Vertical` to flip both.
    pub fn with_flip(&mut self, flip: TileFlip) -> &mut Self {
        self.flip |= flip as u32;
        self
    }

    /// Disable safety check.
    ///
    /// # Important
    /// This is **NOT** recommended. You should only use this if you know what you are doing.
    pub fn with_disabled_safety_check(&mut self) -> &mut Self {
        self.safety_check = false;
        self
    }

    /// Build the tilemap and spawn it into the world.
    /// You can modify the component and insert it back.
    pub fn build(&self, commands: &mut Commands) -> (Entity, Tilemap) {
        if self.safety_check {
            let chunk_count =
                RenderChunkStorage::calculate_render_chunk_count(self.size, self.render_chunk_size);
            if chunk_count.x * chunk_count.y > 100 {
                panic!(
                    "\n============================================\
                    \nYou have too many chunks which may cause performance issue. \
                    Max chunk count: 100, your chunk count: {}x{}={} \
                    \nPlease decrease the map size or increase the render chunk size by calling with_render_chunk_size. \
                    \nCall `with_disabled_safety_check` if you really need to do this.\
                    \n============================================\n",
                    chunk_count.x,
                    chunk_count.y,
                    chunk_count.x * chunk_count.y
                );
            }
        }

        let mut entity = commands.spawn_empty();
        let tilemap = Tilemap {
            tiles: vec![None; (self.size.x * self.size.y) as usize],
            id: entity.id(),
            tile_render_size: self.tile_render_size,
            tile_type: self.tile_type.clone(),
            size: self.size,
            tile_size: self.tile_size,
            render_chunk_size: self.render_chunk_size,
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
    pub(crate) tile_size: UVec2,
    pub(crate) tile_render_size: Vec2,
    pub(crate) render_chunk_size: u32,
    pub(crate) texture: Option<TileTexture>,
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
    pub fn set(&mut self, commands: &mut Commands, tile_builder: TileBuilder) {
        if self.is_out_of_tilemap_uvec(tile_builder.index) {
            return;
        }

        self.set_unchecked(commands, tile_builder);
    }

    pub(crate) fn set_unchecked(&mut self, commands: &mut Commands, tile_builder: TileBuilder) {
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
                self.set_unchecked(commands, builder.clone());
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
                x: (index.x + 0.5) * self.tile_render_size.x + self.translation.x,
                y: (index.y + 0.5) * self.tile_render_size.y + self.translation.y,
            },
            TileType::IsometricDiamond => Vec2 {
                x: (index.x - index.y) / 2. * self.tile_render_size.x + self.translation.x,
                y: (index.x + index.y + 1.) / 2. * self.tile_render_size.y + self.translation.y,
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
