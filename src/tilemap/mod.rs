use bevy::{
    prelude::{
        Assets, Bundle, Commands, Component, Entity, Handle, Image, ResMut, Transform, UVec2, Vec2,
        Vec3, Vec4,
    },
    render::{
        mesh::MeshVertexAttribute,
        render_resource::{FilterMode, TextureUsages, VertexFormat},
    },
    utils::HashMap,
};

use crate::render::{chunk::TileData, texture::TilemapTextureDescriptor};

pub const TILEMAP_MESH_ATTR_GRID_INDEX: MeshVertexAttribute =
    MeshVertexAttribute::new("GridIndex", 14513156146, VertexFormat::Float32x2);
pub const TILEMAP_MESH_ATTR_TEXTURE_INDEX: MeshVertexAttribute =
    MeshVertexAttribute::new("TextureIndex", 186541653135, VertexFormat::Uint32);
pub const TILEMAP_MESH_ATTR_COLOR: MeshVertexAttribute =
    MeshVertexAttribute::new("Color", 85415341854, VertexFormat::Float32x4);

#[derive(Default, PartialEq, Eq, Hash, Clone, Debug)]
pub enum TileType {
    #[default]
    Square,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum TileFlip {
    Horizontal = 1u32 << 0,
    Vertical = 1u32 << 1,
}

#[derive(Clone, Copy)]
pub struct TileBuilder {
    grid_index: UVec2,
    texture_index: u32,
    color: Vec4,
}

impl TileBuilder {
    /// Create a new tile builder.
    pub fn new(grid_index: UVec2, texture_index: u32) -> Self {
        Self {
            grid_index,
            texture_index,
            color: Vec4::ONE,
        }
    }

    pub fn with_color(&mut self, color: Vec4) -> &mut Self {
        self.color = color;
        self
    }

    /// Build the tile and spawn it.
    ///
    /// # Note
    /// DO NOT call this method manually unless you really need to.
    ///
    /// Use `Tilemap::set` instead.
    pub fn build(&self, commands: &mut Commands, tilemap: &Tilemap) -> Entity {
        let render_chunk_index_2d = UVec2::new(
            self.grid_index.x / tilemap.render_chunk_size.x,
            self.grid_index.y / tilemap.render_chunk_size.y,
        );
        let render_chunk_index = {
            if tilemap.size.x % tilemap.render_chunk_size.x == 0 {
                render_chunk_index_2d.y * (tilemap.size.x / tilemap.render_chunk_size.x)
                    + render_chunk_index_2d.x
            } else {
                render_chunk_index_2d.y * (tilemap.size.x / tilemap.render_chunk_size.x + 1)
                    + render_chunk_index_2d.x
            }
        } as usize;
        commands
            .spawn(Tile {
                render_chunk_index,
                tilemap_id: tilemap.id,
                grid_index: self.grid_index,
                texture_index: self.texture_index,
                color: self.color,
            })
            .id()
    }
}

#[derive(Component, Clone, Copy)]
pub struct Tile {
    pub tilemap_id: Entity,
    pub render_chunk_index: usize,
    pub grid_index: UVec2,
    pub texture_index: u32,
    pub color: Vec4,
}

pub struct TilemapBuilder {
    ty: TileType,
    size: UVec2,
    tile_size: UVec2,
    tile_render_size: Vec2,
    render_chunk_size: UVec2,
    texture: Handle<Image>,
    texture_desc: TilemapTextureDescriptor,
    filter_mode: FilterMode,
    transform: Transform,
    flip: u32,
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
            ty,
            size,
            tile_size: UVec2::ZERO,
            texture: Handle::default(),
            texture_desc: TilemapTextureDescriptor::default(),
            tile_render_size,
            render_chunk_size: UVec2::new(16, 16),
            filter_mode: FilterMode::Nearest,
            transform: Transform::IDENTITY,
            flip: 0,
        }
    }

    /// Override z order. Default is 0.
    pub fn with_z_order(&mut self, z_order: f32) -> &mut Self {
        self.transform.translation.z = z_order;
        self
    }

    /// Override render chunk size. Default is 16x16.
    pub fn with_render_chunk_size(&mut self, size: UVec2) -> &mut Self {
        self.render_chunk_size = size;
        self
    }

    /// Override filter mode. Default is nearest.
    pub fn with_filter_mode(&mut self, filter_mode: FilterMode) -> &mut Self {
        self.filter_mode = filter_mode;
        self
    }

    /// Override transform. Default is identity.
    pub fn with_transform(&mut self, transform: Transform) -> &mut Self {
        self.transform = transform;
        self
    }

    pub fn with_texture(
        &mut self,
        texture: Handle<Image>,
        desc: TilemapTextureDescriptor,
    ) -> &mut Self {
        assert!(
            desc.tile_size % desc.tile_count == UVec2::ZERO,
            "The tilemap size must be a multiple of the tile size."
        );

        self.texture = texture;
        self.tile_size = desc.tile_size;
        self.texture_desc = desc;
        self
    }

    /// Align the whole rendered tilemap to the center.
    ///
    /// It will override the `transform` you set.
    pub fn with_center(&mut self, center: Vec2) -> &mut Self {
        self.transform.translation = Vec3::new(
            center.x - self.tile_render_size.x / 2. * self.size.x as f32,
            center.y - self.tile_render_size.y / 2. * self.size.y as f32,
            self.transform.translation.z,
        );
        self
    }

    /// Flip the uv of tiles.
    pub fn with_flip(&mut self, flip: TileFlip) -> &mut Self {
        self.flip |= flip as u32;
        self
    }

    /// Build the tilemap and spawn it into the world.
    pub fn build(&self, commands: &mut Commands) {
        let mut entity = commands.spawn_empty();
        let tilemap = Tilemap {
            tiles: vec![None; self.size.x as usize * self.size.y as usize],
            id: entity.id(),
            tile_render_size: self.tile_render_size,
            tile_type: self.ty.clone(),
            size: self.size,
            tile_size: self.tile_size,
            render_chunk_size: self.render_chunk_size,
            texture: self.texture.clone(),
            texture_desc: self.texture_desc,
            filter_mode: self.filter_mode,
            render_chunks_to_update: HashMap::default(),
            flip: self.flip,
        };
        entity.insert((
            WaitForTextureUsageChange,
            TilemapBundle {
                tilemap,
                transform: self.transform,
            },
        ));
    }
}

#[derive(Component)]
pub struct Tilemap {
    pub(crate) id: Entity,
    pub(crate) tile_type: TileType,
    pub(crate) size: UVec2,
    pub(crate) tile_size: UVec2,
    pub(crate) tile_render_size: Vec2,
    pub(crate) render_chunk_size: UVec2,
    pub(crate) texture: Handle<Image>,
    pub(crate) texture_desc: TilemapTextureDescriptor,
    pub(crate) filter_mode: FilterMode,
    pub(crate) tiles: Vec<Option<Entity>>,
    pub(crate) render_chunks_to_update: HashMap<UVec2, Vec<TileData>>,
    pub(crate) flip: u32,
}

impl Tilemap {
    /// Get a tile.
    pub fn get(&self, grid_index: UVec2) -> Option<Entity> {
        if self.is_out_of_tilemap(grid_index) {
            return None;
        }

        self.get_unchecked(grid_index)
    }

    pub(crate) fn get_unchecked(&self, grid_index: UVec2) -> Option<Entity> {
        self.tiles[grid_index.y as usize * self.size.x as usize + grid_index.x as usize]
    }

    /// Set a tile.
    ///
    /// Overwrites the tile if it already exists.
    pub fn set(&mut self, commands: &mut Commands, tile_builder: TileBuilder) {
        if self.is_out_of_tilemap(tile_builder.grid_index) {
            return;
        }

        self.set_unchecked(commands, tile_builder);
    }

    pub(crate) fn set_unchecked(&mut self, commands: &mut Commands, tile_builder: TileBuilder) {
        let index = (tile_builder.grid_index.y * self.size.x + tile_builder.grid_index.x) as usize;
        if let Some(previous) = self.tiles[index] {
            commands.entity(previous).despawn();
        }
        let new_tile = tile_builder.build(commands, self);
        self.tiles[index] = Some(new_tile);
    }

    /// Remove a tile.
    pub fn remove(&mut self, commands: &mut Commands, grid_index: UVec2) {
        if self.is_out_of_tilemap(grid_index) || self.get(grid_index).is_none() {
            return;
        }

        self.remove_unchecked(commands, grid_index);
    }

    pub(crate) fn remove_unchecked(&mut self, commands: &mut Commands, grid_index: UVec2) {
        let index = (grid_index.y * self.size.x + grid_index.x) as usize;
        commands.entity(self.tiles[index].unwrap()).despawn();
        self.tiles[index] = None;
    }

    /// Fill a rectangle area with tiles.
    ///
    /// You don't need to assign the `grid_index` in the `tile_builder`.
    pub fn fill_rect(
        &mut self,
        commands: &mut Commands,
        origin: UVec2,
        extent: UVec2,
        tile_builder: &TileBuilder,
    ) {
        let dst = origin + extent - UVec2::ONE;
        assert!(
            !(self.is_out_of_tilemap(origin) || self.is_out_of_tilemap(dst)),
            "Part of the area is out of the tilemap! Max size: {:?}",
            self.size
        );

        let mut builder = tile_builder.clone();
        for y in origin.y..=dst.y {
            for x in origin.x..=dst.x {
                builder.grid_index = UVec2::new(x, y);
                self.set_unchecked(commands, builder.clone());
            }
        }
    }

    pub fn is_out_of_tilemap(&self, grid_index: UVec2) -> bool {
        grid_index.x >= self.size.x || grid_index.y >= self.size.y
    }

    /// Bevy doesn't set the `COPY_SRC` usage for images by default, so we need to do it manually.
    pub(crate) fn set_usage(
        &mut self,
        commands: &mut Commands,
        image_assets: &mut ResMut<Assets<Image>>,
    ) {
        let Some(image) = image_assets.get(&self.texture) else {
            return;
        };

        if !image
            .texture_descriptor
            .usage
            .contains(TextureUsages::COPY_SRC)
        {
            image_assets
                .get_mut(&self.texture)
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

#[derive(Bundle)]
pub struct TilemapBundle {
    pub tilemap: Tilemap,
    pub transform: Transform,
}

#[derive(Component)]
pub struct WaitForTextureUsageChange;
