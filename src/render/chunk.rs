use bevy::{
    prelude::{Commands, Entity, Handle, Image, Mesh, Query, Res, ResMut, Resource, UVec2, Vec3},
    render::{
        mesh::{GpuBufferInfo, GpuMesh, Indices},
        render_resource::{BufferInitDescriptor, BufferUsages, IndexFormat, PrimitiveTopology},
        renderer::RenderDevice,
    },
    utils::HashMap,
};

use crate::tilemap::{
    Tile, TileType, Tilemap, TILEMAP_MESH_ATTR_GRID_INDEX, TILEMAP_MESH_ATTR_TEXTURE_INDEX,
};

use super::extract::{ExtractedTile, ExtractedTilemap};

#[derive(Clone, Debug)]
pub struct TileData {
    pub grid_index: UVec2,
    pub texture_index: u32,
}

#[derive(Clone)]
pub struct TilemapRenderChunk {
    pub dirty_mesh: bool,
    pub tile_type: TileType,
    pub size: UVec2,
    pub texture: Handle<Image>,
    pub tiles: Vec<Option<TileData>>,
    pub mesh: Mesh,
    pub gpu_mesh: Option<GpuMesh>,
}

impl TilemapRenderChunk {
    pub fn new(tilemap: &ExtractedTilemap) -> Self {
        TilemapRenderChunk {
            size: tilemap.render_chunk_size,
            tile_type: tilemap.tile_type.clone(),
            texture: tilemap.texture.clone(),
            tiles: vec![None; tilemap.render_chunk_size.length_squared() as usize],
            mesh: Mesh::new(PrimitiveTopology::TriangleList),
            gpu_mesh: None,
            dirty_mesh: true,
        }
    }

    /// Update the raw mesh for GPU processing.
    pub fn update_mesh(&mut self, render_device: &Res<RenderDevice>) {
        if !self.dirty_mesh {
            return;
        }

        // let mut v_index = 0;
        // let len = self.tiles.len();
        // let mut positions = Vec::with_capacity(len * 4);
        // let mut texture_indices = Vec::with_capacity(len * 4);
        // let mut grid_indices = Vec::with_capacity(len * 4);
        // let mut vertex_indices = Vec::with_capacity(len * 6);

        // for tile_data in self.tiles.iter() {
        //     if let Some(tile) = tile_data {
        //         positions.extend_from_slice(&[Vec3::ZERO, Vec3::ZERO, Vec3::ZERO, Vec3::ZERO]);

        //         grid_indices.extend_from_slice(&[
        //             tile.grid_index,
        //             tile.grid_index,
        //             tile.grid_index,
        //             tile.grid_index,
        //         ]);

        //         texture_indices.extend_from_slice(&[
        //             tile.texture_index,
        //             tile.texture_index,
        //             tile.texture_index,
        //             tile.texture_index,
        //         ]);

        //         vertex_indices.extend_from_slice(&[
        //             v_index,
        //             v_index + 2,
        //             v_index + 1,
        //             v_index,
        //             v_index + 3,
        //             v_index + 2,
        //         ]);

        //         v_index += 4;
        //     }
        // }

        // self.mesh
        //     .insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        // self.mesh
        //     .insert_attribute(TILEMAP_MESH_ATTR_GRID_INDEX, grid_indices);
        // self.mesh
        //     .insert_attribute(TILEMAP_MESH_ATTR_TEXTURE_INDEX, texture_indices);
        // self.mesh.set_indices(Some(Indices::U32(vertex_indices)));

        self.mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vec![
                Vec3::X * 150.,
                Vec3::NEG_X * 150.,
                Vec3::Y * 150.,
                Vec3::NEG_Y * 150.,
            ],
        );
        self.mesh.insert_attribute(
            TILEMAP_MESH_ATTR_GRID_INDEX,
            vec![UVec2::ZERO, UVec2::ZERO, UVec2::ZERO, UVec2::ZERO],
        );
        self.mesh
            .insert_attribute(TILEMAP_MESH_ATTR_TEXTURE_INDEX, vec![0u32, 0u32, 0u32, 0u32]);
        self.mesh
            .set_indices(Some(Indices::U32(vec![0u32, 1u32, 2u32, 0u32, 3u32, 2u32])));

        let mesh_vert_cout = self.mesh.count_vertices() as u32;

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
                    count: mesh_vert_cout,
                    index_format: IndexFormat::Uint32,
                });

        self.gpu_mesh = Some(GpuMesh {
            vertex_buffer,
            vertex_count: mesh_vert_cout,
            morph_targets: None,
            buffer_info,
            primitive_topology: PrimitiveTopology::TriangleList,
            layout: self.mesh.get_mesh_vertex_buffer_layout(),
        });

        self.dirty_mesh = false;
    }

    /// Set a tile in the chunk. Overwrites the previous tile.
    pub fn set_tile(&mut self, tile: &ExtractedTile) {
        let index = (tile.grid_index.y * self.size.x + tile.grid_index.x) as usize;
        self.tiles[index] = Some(TileData {
            grid_index: tile.grid_index,
            texture_index: tile.texture_index,
        });
        self.dirty_mesh = true;
    }

    /// Set multiple tiles in the chunk. Overwrites the previous tiles.
    pub(crate) fn set_tiles(&mut self, tiles: &Vec<TileData>) {
        tiles.iter().for_each(|elem| {
            let index = (elem.grid_index.y * self.size.x + elem.grid_index.x) as usize;
            self.tiles[index] = Some(elem.clone());
        });
        self.dirty_mesh = true;
    }
}

#[derive(Resource, Default)]
pub struct RenderChunkStorage {
    value: HashMap<Entity, Vec<Option<TilemapRenderChunk>>>,
}

impl RenderChunkStorage {
    /// Insert new render chunks into the storage for a tilemap.
    pub fn insert_tilemap(&mut self, tilemap: &ExtractedTilemap) {
        let amount = Self::calculate_render_chunk_storage_size(tilemap);
        self.value.insert(tilemap.id, vec![None; amount]);
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

    pub fn add_tiles_with_query(
        &mut self,
        tilemaps_query: &Query<&ExtractedTilemap>,
        tiles_query: &Query<&ExtractedTile>,
    ) {
        for tile in tiles_query.iter() {
            if let Some(chunks) = self.value.get_mut(&tile.tilemap) {
                if let Some(chunk) = chunks.get_mut(tile.render_chunk_index).unwrap() {
                    chunk.set_tile(tile);
                } else {
                    let tilemap = tilemaps_query.get(tile.tilemap).unwrap();
                    chunks[tile.render_chunk_index] = Some(TilemapRenderChunk::new(tilemap));
                }
            } else {
                let tilemap = tilemaps_query.get(tile.tilemap).unwrap();
                self.insert_tilemap(tilemap);
                self.value.get_mut(&tile.tilemap).unwrap()[tile.render_chunk_index] =
                    Some(TilemapRenderChunk::new(tilemap));
            }
        }
    }

    pub fn get(&self, tilemap: Entity) -> Option<&Vec<Option<TilemapRenderChunk>>> {
        self.value.get(&tilemap)
    }

    fn calculate_render_chunk_storage_size(tilemap: &ExtractedTilemap) -> usize {
        UVec2::new(
            {
                if tilemap.size.x % tilemap.render_chunk_size.x == 0 {
                    tilemap.size.x / tilemap.render_chunk_size.x
                } else {
                    tilemap.size.x / tilemap.render_chunk_size.x + 1
                }
            },
            {
                if tilemap.size.y % tilemap.render_chunk_size.y == 0 {
                    tilemap.size.y / tilemap.render_chunk_size.y
                } else {
                    tilemap.size.y / tilemap.render_chunk_size.y + 1
                }
            },
        )
        .length_squared() as usize
    }
}
