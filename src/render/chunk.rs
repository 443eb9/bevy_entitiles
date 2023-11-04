use bevy::{
    prelude::{Changed, Entity, Mesh, Or, Query, Res, Resource, UVec2, Vec2, Vec3, Vec4, Without},
    render::{
        mesh::{GpuBufferInfo, GpuMesh, Indices},
        render_resource::{BufferInitDescriptor, BufferUsages, IndexFormat, PrimitiveTopology},
        renderer::RenderDevice,
    },
    utils::HashMap,
};

use crate::{
    math::geometry::AabbBox2d,
    tilemap::{
        TileTexture, TileType, TILEMAP_MESH_ATTR_COLOR, TILEMAP_MESH_ATTR_GRID_INDEX,
        TILEMAP_MESH_ATTR_TEXTURE_INDEX,
    },
};

use super::{
    culling::Visible,
    extract::{ExtractedTile, ExtractedTilemap},
};

#[derive(Clone, Debug)]
pub struct TileData {
    pub grid_index: UVec2,
    pub texture_index: u32,
    pub color: Vec4,
}

#[derive(Clone)]
pub struct TilemapRenderChunk {
    pub visible: bool,
    pub index: UVec2,
    pub dirty_mesh: bool,
    pub tile_type: TileType,
    pub size: u32,
    pub texture: Option<TileTexture>,
    pub tiles: Vec<Option<TileData>>,
    pub mesh: Mesh,
    pub gpu_mesh: Option<GpuMesh>,
    pub aabb: AabbBox2d,
}

impl TilemapRenderChunk {
    pub fn from_grid_index(grid_index: UVec2, tilemap: &ExtractedTilemap) -> Self {
        let index = grid_index / tilemap.render_chunk_size;
        TilemapRenderChunk {
            visible: true,
            index,
            size: tilemap.render_chunk_size,
            tile_type: tilemap.tile_type.clone(),
            texture: tilemap.texture.clone(),
            tiles: vec![None; (tilemap.render_chunk_size * tilemap.render_chunk_size) as usize],
            mesh: Mesh::new(PrimitiveTopology::TriangleList),
            gpu_mesh: None,
            dirty_mesh: true,
            aabb: AabbBox2d::from_chunk(index, tilemap),
        }
    }

    /// Update the raw mesh for GPU processing.
    pub fn update_mesh(&mut self, render_device: &Res<RenderDevice>) {
        if !self.dirty_mesh {
            return;
        }

        let mut v_index = 0;
        let len = self.tiles.len();
        let mut positions = Vec::with_capacity(len * 4);
        let mut texture_indices = Vec::with_capacity(len * 4);
        let mut grid_indices = Vec::with_capacity(len * 4);
        let mut vertex_indices = Vec::with_capacity(len * 6);
        let mut color = Vec::with_capacity(len * 4);

        for tile_data in self.tiles.iter() {
            if let Some(tile) = tile_data {
                positions.extend_from_slice(&[Vec3::ZERO, Vec3::ZERO, Vec3::ZERO, Vec3::ZERO]);

                let grid_index_float =
                    Vec2::new(tile.grid_index.x as f32, tile.grid_index.y as f32);
                grid_indices.extend_from_slice(&[
                    grid_index_float,
                    grid_index_float,
                    grid_index_float,
                    grid_index_float,
                ]);

                texture_indices.extend_from_slice(&[
                    tile.texture_index,
                    tile.texture_index,
                    tile.texture_index,
                    tile.texture_index,
                ]);

                vertex_indices.extend_from_slice(&[
                    v_index,
                    v_index + 1,
                    v_index + 3,
                    v_index + 1,
                    v_index + 2,
                    v_index + 3,
                ]);

                v_index += 4;

                color.extend_from_slice(&[tile.color, tile.color, tile.color, tile.color]);
            }
        }

        self.mesh
            .insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        self.mesh
            .insert_attribute(TILEMAP_MESH_ATTR_GRID_INDEX, grid_indices);
        self.mesh
            .insert_attribute(TILEMAP_MESH_ATTR_TEXTURE_INDEX, texture_indices);
        self.mesh.insert_attribute(TILEMAP_MESH_ATTR_COLOR, color);
        self.mesh.set_indices(Some(Indices::U32(vertex_indices)));

        let mesh_vert_count = self.mesh.count_vertices() as u32;
        let mesh_indices_count = self.mesh.indices().unwrap().len() as u32;

        let vertex_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("tilemap_vertex_buffer"),
            contents: &self.mesh.get_vertex_buffer_data(),
            usage: BufferUsages::VERTEX,
        });

        let buffer_info =
            self.mesh
                .get_index_buffer_bytes()
                .map_or(GpuBufferInfo::NonIndexed, |data| GpuBufferInfo::Indexed {
                    buffer: render_device.create_buffer_with_data(&BufferInitDescriptor {
                        label: Some("tilemap_index_buffer"),
                        contents: data,
                        usage: BufferUsages::INDEX,
                    }),
                    count: mesh_indices_count,
                    index_format: IndexFormat::Uint32,
                });

        self.gpu_mesh = Some(GpuMesh {
            vertex_buffer,
            vertex_count: mesh_vert_count,
            morph_targets: None,
            buffer_info,
            primitive_topology: PrimitiveTopology::TriangleList,
            layout: self.mesh.get_mesh_vertex_buffer_layout(),
        });

        self.dirty_mesh = false;
    }

    /// Set a tile in the chunk. Overwrites the previous tile.
    pub fn set_tile(&mut self, grid_index: UVec2, tile: &ExtractedTile) {
        let index = (grid_index.y * self.size + grid_index.x) as usize;
        self.tiles[index] = Some(TileData {
            grid_index: tile.grid_index,
            texture_index: tile.texture_index,
            color: tile.color,
        });
        self.dirty_mesh = true;
    }
}

#[derive(Resource, Default)]
pub struct RenderChunkStorage {
    pub(crate) value: HashMap<Entity, Vec<Option<TilemapRenderChunk>>>,
}

impl RenderChunkStorage {
    /// Insert new render chunks into the storage for a tilemap.
    pub fn insert_tilemap(&mut self, tilemap: &ExtractedTilemap) {
        let amount = Self::calculate_render_chunk_count(tilemap);
        self.value
            .insert(tilemap.id, vec![None; (amount.x * amount.y) as usize]);
    }

    /// Update the mesh for all chunks of a tilemap.
    pub fn prepare_chunks(
        &mut self,
        tilemap: &ExtractedTilemap,
        render_device: &Res<RenderDevice>,
    ) {
        if let Some(chunks) = self.value.get_mut(&tilemap.id) {
            for chunk in chunks.iter_mut() {
                if let Some(c) = chunk {
                    c.update_mesh(render_device);
                }
            }
        }
    }

    /// Add tiles to the storage from a query.
    pub fn add_tiles_with_query(
        &mut self,
        tilemaps_query: &Query<&ExtractedTilemap, Without<Visible>>,
        changed_tiles_query: &Query<&ExtractedTile, Or<(Changed<ExtractedTile>,)>>,
    ) {
        for tile in changed_tiles_query.iter() {
            let tilemap = tilemaps_query.get(tile.tilemap).unwrap();

            let chunks = {
                if !self.value.contains_key(&tile.tilemap) {
                    self.insert_tilemap(tilemap)
                }
                self.value.get_mut(&tile.tilemap).unwrap()
            };

            let chunk = {
                if chunks[tile.render_chunk_index].is_none() {
                    chunks[tile.render_chunk_index] = Some(TilemapRenderChunk::from_grid_index(
                        tile.grid_index,
                        tilemap,
                    ));
                }
                chunks.get_mut(tile.render_chunk_index).unwrap()
            };

            let c = {
                if chunk.is_none() {
                    chunk.replace(TilemapRenderChunk::from_grid_index(
                        tile.grid_index,
                        tilemap,
                    ));
                };
                chunk.as_mut().unwrap()
            };

            let grid_index = tile.grid_index % tilemap.render_chunk_size;
            c.set_tile(grid_index, tile);
        }
    }

    pub fn get(&self, tilemap: Entity) -> Option<&Vec<Option<TilemapRenderChunk>>> {
        self.value.get(&tilemap)
    }

    pub fn get_mut(&mut self, tilemap: Entity) -> Option<&mut Vec<Option<TilemapRenderChunk>>> {
        self.value.get_mut(&tilemap)
    }

    pub fn calculate_render_chunk_count(tilemap: &ExtractedTilemap) -> UVec2 {
        UVec2::new(
            {
                if tilemap.size.x % tilemap.render_chunk_size == 0 {
                    tilemap.size.x / tilemap.render_chunk_size
                } else {
                    tilemap.size.x / tilemap.render_chunk_size + 1
                }
            },
            {
                if tilemap.size.y % tilemap.render_chunk_size == 0 {
                    tilemap.size.y / tilemap.render_chunk_size
                } else {
                    tilemap.size.y / tilemap.render_chunk_size + 1
                }
            },
        )
    }
}
