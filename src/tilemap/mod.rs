use bevy::{
    prelude::{Bundle, Commands, Component, Entity, Handle, Image, Transform, UVec2},
    render::{
        mesh::MeshVertexAttribute,
        render_resource::{FilterMode, VertexFormat},
    },
    utils::HashMap,
};

use crate::render::chunk::TileData;

pub const TILEMAP_MESH_ATTR_GRID_INDEX: MeshVertexAttribute =
    MeshVertexAttribute::new("GridIndex", 14513156146, VertexFormat::Uint32x2);
pub const TILEMAP_MESH_ATTR_TEXTURE_INDEX: MeshVertexAttribute =
    MeshVertexAttribute::new("TextureIndex", 186541653135, VertexFormat::Uint32);

#[derive(Default, PartialEq, Eq, Hash, Clone)]
pub enum TileType {
    #[default]
    Square,
}

pub struct TileBuilder {
    grid_index: UVec2,
    texture_index: u32,
}

impl TileBuilder {
    /// Create a new tile builder.
    pub fn new(grid_index: UVec2, texture_index: u32) -> Self {
        Self {
            grid_index,
            texture_index,
        }
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
        commands
            .spawn(Tile {
                render_chunk_index: (render_chunk_index_2d.y * tilemap.size.x
                    + render_chunk_index_2d.x) as usize,
                tilemap_id: tilemap.id,
                grid_index: self.grid_index,
                texture_index: self.texture_index,
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
}

pub struct TilemapBuilder {
    ty: TileType,
    size: UVec2,
    tile_size: UVec2,
    render_chunk_size: UVec2,
    texture: Handle<Image>,
    filter_mode: FilterMode,
    z_order: f32,
    transform: Transform,
}

impl TilemapBuilder {
    /// Create a new tilemap builder.
    ///
    /// Notice that the id should **must** be unique. Randomly generated ids are recommended.
    pub fn new(ty: TileType, size: UVec2, tile_size: UVec2, texture: Handle<Image>) -> Self {
        Self {
            ty,
            size,
            tile_size,
            render_chunk_size: UVec2::new(16, 16),
            texture,
            filter_mode: FilterMode::Nearest,
            z_order: 0.,
            transform: Transform::IDENTITY,
        }
    }

    /// Override z order. Default is 0.
    pub fn with_z_order(mut self, z_order: f32) -> Self {
        self.z_order = z_order;
        self
    }

    /// Override render chunk size. Default is 16x16.
    pub fn with_render_chunk_size(mut self, size: UVec2) -> Self {
        self.render_chunk_size = size;
        self
    }

    /// Override filter mode. Default is nearest.
    pub fn with_filter_mode(mut self, filter_mode: FilterMode) -> Self {
        self.filter_mode = filter_mode;
        self
    }

    /// Override transform. Default is identity.
    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
    }

    /// Build the tilemap and spawn it into the world.
    pub fn build(&self, commands: &mut Commands) {
        let mut entity = commands.spawn_empty();
        let tilemap = Tilemap {
            tiles: vec![None; self.size.x as usize * self.size.y as usize],
            id: entity.id(),
            tile_type: self.ty.clone(),
            size: self.size,
            tile_size: self.tile_size,
            render_chunk_size: self.render_chunk_size,
            texture: self.texture.clone(),
            filter_mode: self.filter_mode,
            z_order: self.z_order,
            render_chunks_to_update: HashMap::default(),
        };
        entity.insert(TilemapBundle {
            tilemap,
            transform: self.transform,
        });
    }
}

#[derive(Component)]
pub struct Tilemap {
    pub(crate) id: Entity,
    pub(crate) tile_type: TileType,
    pub(crate) size: UVec2,
    pub(crate) tile_size: UVec2,
    pub(crate) render_chunk_size: UVec2,
    pub(crate) texture: Handle<Image>,
    pub(crate) z_order: f32,
    pub(crate) filter_mode: FilterMode,
    pub(crate) tiles: Vec<Option<Entity>>,
    pub(crate) render_chunks_to_update: HashMap<UVec2, Vec<TileData>>,
}

impl Tilemap {
    /// Get a tile.
    pub fn get(&self, grid_index: UVec2) -> Option<Entity> {
        let tile_index = (grid_index.y * self.size.x + grid_index.x) as usize;
        if tile_index > self.tiles.capacity() {
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
        let tile_index =
            (tile_builder.grid_index.y * self.size.x + tile_builder.grid_index.x) as usize;
        if tile_index > self.tiles.capacity() {
            return;
        }

        self.set_unchecked(commands, tile_builder);
    }

    pub(crate) fn set_unchecked(&mut self, commands: &mut Commands, tile_builder: TileBuilder) {
        let new_tile = tile_builder.build(commands, self);
        let index = (tile_builder.grid_index.y * self.size.x + tile_builder.grid_index.x) as usize;
        self.tiles[index] = Some(new_tile);
    }
}

#[derive(Bundle)]
pub struct TilemapBundle {
    pub tilemap: Tilemap,
    pub transform: Transform,
}
