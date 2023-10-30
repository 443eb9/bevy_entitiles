use bevy::{
    prelude::{
        Handle, Image,
        Transform, UVec2, Vec3, Component,
    },
    render::{mesh::MeshVertexAttribute, render_resource::{VertexFormat, FilterMode}},
};

use crate::render::RenderChunkStorage;

pub const TILEMAP_MESH_ATTR_POSITION: MeshVertexAttribute =
    MeshVertexAttribute::new("Position", 14513156146, VertexFormat::Float32x3);
pub const TILEMAP_MESH_ATTR_TEXTURE_INDEX: MeshVertexAttribute =
    MeshVertexAttribute::new("TextureIndex", 186541653135, VertexFormat::Uint32);

#[derive(Default, PartialEq, Eq, Hash, Clone)]
pub enum TileType {
    #[default]
    Square,
}

#[derive(Clone, Copy)]
pub struct Tile {
    pub world_pos: Vec3,
    pub coordinate: UVec2,
    pub texture_index: u32,
}

pub struct TilemapBuilder {
    id: u32,
    transform: Transform,
    ty: TileType,
    size: UVec2,
    tile_size: UVec2,
    render_chunk_size: UVec2,
    texture: Handle<Image>,
    filter_mode: FilterMode,
    z_order: f32,
}

impl TilemapBuilder {
    /// Create a new tilemap builder.
    ///
    /// Notice that the id should **must** be unique. Randomly generated ids are recommended.
    pub fn new(
        id: u32,
        ty: TileType,
        size: UVec2,
        tile_size: UVec2,
        texture: Handle<Image>,
    ) -> Self {
        Self {
            id,
            ty,
            size,
            tile_size,
            transform: Transform::default(),
            render_chunk_size: UVec2::new(16, 16),
            texture,
            filter_mode: FilterMode::Nearest,
            z_order: 0.,
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

    /// Override transform. Default is identity.
    pub fn with_translate(mut self, translate: Vec3) -> Self {
        self.transform.translation = translate;
        self
    }

    /// Override transform. Default is identity.
    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
    }

    /// Override filter mode. Default is nearest.
    pub fn with_filter_mode(mut self, filter_mode: FilterMode) -> Self {
        self.filter_mode = filter_mode;
        self
    }
}

#[derive(Component)]
pub struct Tilemap {
    pub(crate) id: u32,
    pub(crate) transform: Transform,
    pub(crate) tile_type: TileType,
    pub(crate) size: UVec2,
    pub(crate) tile_size: UVec2,
    pub(crate) render_chunk_size: UVec2,
    pub(crate) texture: Handle<Image>,
    pub(crate) z_order: f32,
    pub(crate) filter_mode: FilterMode,
    pub(crate) tiles: Vec<Option<Tile>>,
}

impl From<TilemapBuilder> for Tilemap {
    fn from(value: TilemapBuilder) -> Self {
        Tilemap {
            tiles: vec![None; value.size.x as usize * value.size.y as usize],
            id: value.id,
            transform: value.transform,
            tile_type: value.ty,
            size: value.size,
            tile_size: value.tile_size,
            render_chunk_size: value.render_chunk_size,
            texture: value.texture,
            filter_mode: value.filter_mode,
            z_order: value.z_order,
        }
    }
}

impl Tilemap {
    pub fn get(&self, coordinate: UVec2) -> Option<Tile> {
        self.tiles
            .get(coordinate.y as usize * self.size.x as usize + coordinate.x as usize)
            .copied()
            .flatten()
    }

    pub fn get_unchecked(&self, coordinate: UVec2) -> Option<Tile> {
        self.tiles[coordinate.y as usize * self.size.x as usize + coordinate.x as usize]
    }

    pub fn set(
        &mut self,
        coordinate: UVec2,
        texture_index: u32,
        render_chunk_storage: &mut RenderChunkStorage,
    ) {
        let index = (coordinate.y * self.size.x + coordinate.x) as usize;
        if index > self.tiles.capacity() {
            return;
        }

        self.set_unchecked(coordinate, texture_index, render_chunk_storage);
    }

    pub fn set_unchecked(
        &mut self,
        coordinate: UVec2,
        texture_index: u32,
        render_chunk_storage: &mut RenderChunkStorage,
    ) {
        let new_tile = Tile {
            world_pos: self.transform.transform_point(Vec3::new(
                (coordinate.x * self.tile_size.x) as f32,
                (coordinate.y * self.tile_size.y) as f32,
                self.z_order,
            )),
            coordinate,
            texture_index,
        };

        self.tiles[(coordinate.y * self.size.x + coordinate.x) as usize] = Some(new_tile);

        if render_chunk_storage.value.contains_key(&self.id) {
            render_chunk_storage.value.insert(
                self.id,
                Vec::with_capacity(self.calculate_render_chunk_storage_size()),
            );
        }

        render_chunk_storage.value.get_mut(&self.id).unwrap()
            [(coordinate.x + coordinate.y * self.size.x) as usize]
            .update_tiles(new_tile);
    }

    /// Calculate the world position of a tile coordinate.
    pub fn calculate_tile_world_pos(&self, coordinate: UVec2) -> Vec3 {
        self.transform.transform_point(Vec3::new(
            (coordinate.x * self.tile_size.x) as f32,
            (coordinate.y * self.tile_size.y) as f32,
            self.z_order,
        ))
    }

    fn calculate_render_chunk_storage_size(&self) -> usize {
        UVec2::new(
            {
                if self.size.x % self.render_chunk_size.x == 0 {
                    self.size.x / self.render_chunk_size.x
                } else {
                    self.size.x / self.render_chunk_size.x + 1
                }
            },
            {
                if self.size.y % self.render_chunk_size.y == 0 {
                    self.size.y / self.render_chunk_size.y
                } else {
                    self.size.y / self.render_chunk_size.y + 1
                }
            },
        )
        .length_squared() as usize
    }
}
