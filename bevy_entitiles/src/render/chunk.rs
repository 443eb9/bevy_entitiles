use bevy::{
    prelude::{Entity, Mesh, Query, Resource, UVec2, Vec3, Vec4},
    render::{
        mesh::{GpuBufferInfo, GpuMesh, Indices},
        render_resource::{BufferInitDescriptor, BufferUsages, IndexFormat, PrimitiveTopology},
        renderer::RenderDevice,
    },
    utils::HashMap,
};

use crate::{math::aabb::AabbBox2d, tilemap::tile::TileType, MAX_LAYER_COUNT};

use super::{
    extract::{ExtractedTile, ExtractedTilemap},
    texture::TilemapTexture,
    TILEMAP_MESH_ATTR_ATLAS_INDICES, TILEMAP_MESH_ATTR_COLOR, TILEMAP_MESH_ATTR_INDEX,
    TILEMAP_MESH_ATTR_RD_SIZE,
};

#[derive(Clone)]
pub struct TileData {
    // when the third component of index is negative,
    // it means this tile is a animated tile
    // so the first component of texture_indices is the index of the animation
    pub index: Vec3,
    pub texture_indices: [i32; MAX_LAYER_COUNT],
    pub color: Vec4,
}

#[derive(Clone)]
pub struct TilemapRenderChunk {
    pub visible: bool,
    pub index: UVec2,
    pub dirty_mesh: bool,
    pub tile_type: TileType,
    pub size: u32,
    pub texture: Option<TilemapTexture>,
    pub tiles: Vec<Option<TileData>>,
    pub mesh: Mesh,
    pub gpu_mesh: Option<GpuMesh>,
    pub aabb: AabbBox2d,
    pub flip: u32,
}

impl TilemapRenderChunk {
    pub fn from_index(index: UVec2, tilemap: &ExtractedTilemap) -> Self {
        let idx = index / tilemap.render_chunk_size;
        TilemapRenderChunk {
            visible: true,
            index: idx,
            size: tilemap.render_chunk_size,
            tile_type: tilemap.tile_type,
            texture: tilemap.texture.clone(),
            flip: tilemap.flip,
            tiles: vec![None; (tilemap.render_chunk_size * tilemap.render_chunk_size) as usize],
            mesh: Mesh::new(PrimitiveTopology::TriangleList),
            gpu_mesh: None,
            dirty_mesh: true,
            aabb: AabbBox2d::from_chunk(idx, tilemap),
        }
    }

    /// Update the raw mesh for GPU processing.
    pub fn update_mesh(&mut self, render_device: &RenderDevice) {
        if !self.dirty_mesh {
            return;
        }
        let (tile_uvs, is_uniform, is_pure_color) = {
            if let Some(texture) = &self.texture {
                (Some(&texture.desc.tiles_uv), texture.desc.is_uniform, false)
            } else {
                (None, true, true)
            }
        };

        let mut v_index = 0;
        let len = self.tiles.len();

        let mut positions = Vec::with_capacity(len * 4);
        let mut atlas_indices = Vec::with_capacity(len * 4);
        let mut grid_indices = Vec::with_capacity(len * 4);
        let mut vertex_indices = Vec::with_capacity(len * 6);
        let mut color = Vec::with_capacity(len * 4);
        let mut tile_render_size = Vec::new();
        if !is_uniform {
            tile_render_size = Vec::with_capacity(len * 4);
        }

        for tile_data in self.tiles.iter() {
            if let Some(tile) = tile_data {
                grid_indices.extend_from_slice(&[tile.index, tile.index, tile.index, tile.index]);

                if !is_pure_color {
                    atlas_indices.extend_from_slice(&[
                        tile.texture_indices,
                        tile.texture_indices,
                        tile.texture_indices,
                        tile.texture_indices,
                    ]);

                    let t_uvs = tile_uvs.unwrap();
                    if !is_uniform {
                        // non uniform tilemaps are not allowed to have multiple layers
                        let size = t_uvs[tile.texture_indices[0] as usize].render_size();
                        tile_render_size.extend_from_slice(&[size, size, size, size]);
                    }
                }

                let pos = Vec3::ZERO;
                positions.extend_from_slice(&[pos, pos, pos, pos]);

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
            .insert_attribute(TILEMAP_MESH_ATTR_INDEX, grid_indices);
        if !is_pure_color {
            self.mesh
                .insert_attribute(TILEMAP_MESH_ATTR_ATLAS_INDICES, atlas_indices);
        }
        self.mesh.insert_attribute(TILEMAP_MESH_ATTR_COLOR, color);
        if !is_uniform {
            self.mesh
                .insert_attribute(TILEMAP_MESH_ATTR_RD_SIZE, tile_render_size);
        }
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
    pub fn set_tile(&mut self, index: UVec2, tile: &ExtractedTile) {
        let index = (index.y * self.size + index.x) as usize;

        // TODO fix this. this allows the tile sort by y axis. But this approach is not good.
        let index = self.tiles.len() - index - 1;

        let (tile_index, texture_indices) = {
            if let Some(anim) = tile.anim.as_ref() {
                let mut tex_idxes = tile.texture_indices;
                tex_idxes[0] = anim.sequence_index as i32;
                (tile.index.as_vec2().extend(-1.), tex_idxes)
            } else {
                (tile.index.as_vec2().extend(1.), tile.texture_indices)
            }
        };

        self.tiles[index] = Some(TileData {
            index: tile_index,
            texture_indices,
            color: tile.color,
        });
        self.dirty_mesh = true;
    }
}

#[derive(Resource, Default)]
pub struct RenderChunkStorage {
    pub(crate) value: HashMap<Entity, (UVec2, Vec<Option<TilemapRenderChunk>>)>,
}

impl RenderChunkStorage {
    /// Insert new render chunks into the storage for a tilemap.
    pub fn insert_tilemap(&mut self, tilemap: &ExtractedTilemap) {
        let amount = Self::calculate_render_chunk_count(tilemap.size, tilemap.render_chunk_size);
        self.value.insert(
            tilemap.id,
            (amount, vec![None; (amount.x * amount.y) as usize]),
        );
    }

    /// Update the mesh for all chunks of a tilemap.
    pub fn prepare_chunks(&mut self, tilemap: &ExtractedTilemap, render_device: &RenderDevice) {
        if let Some(chunks) = self.value.get_mut(&tilemap.id) {
            for chunk in chunks.1.iter_mut() {
                if let Some(c) = chunk {
                    c.update_mesh(render_device);
                }
            }
        }
    }

    /// Add tiles to the storage from a query.
    pub fn add_tiles_with_query(
        &mut self,
        tilemaps_query: &Query<&ExtractedTilemap>,
        changed_tiles_query: &Query<&mut ExtractedTile>,
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
                if chunks.1[tile.render_chunk_index].is_none() {
                    chunks.1[tile.render_chunk_index] =
                        Some(TilemapRenderChunk::from_index(tile.index, tilemap));
                }
                chunks.1.get_mut(tile.render_chunk_index).unwrap()
            };

            let c = {
                if chunk.is_none() {
                    chunk.replace(TilemapRenderChunk::from_index(tile.index, tilemap));
                };
                chunk.as_mut().unwrap()
            };

            let index = tile.index % tilemap.render_chunk_size;
            c.set_tile(index, tile);
        }
    }

    pub fn get_chunks(&self, tilemap: Entity) -> Option<&Vec<Option<TilemapRenderChunk>>> {
        if let Some(chunks) = self.value.get(&tilemap) {
            Some(&chunks.1)
        } else {
            None
        }
    }

    pub fn get_chunks_mut(
        &mut self,
        tilemap: Entity,
    ) -> Option<&mut Vec<Option<TilemapRenderChunk>>> {
        if let Some(chunks) = self.value.get_mut(&tilemap) {
            Some(chunks.1.as_mut())
        } else {
            None
        }
    }

    pub fn get_storage_size(&self, tilemap: Entity) -> Option<UVec2> {
        if let Some(chunks) = self.value.get(&tilemap) {
            Some(chunks.0)
        } else {
            None
        }
    }

    pub fn calculate_render_chunk_count(map_size: UVec2, render_chunk_size: u32) -> UVec2 {
        UVec2::new(
            {
                if map_size.x % render_chunk_size == 0 {
                    map_size.x / render_chunk_size
                } else {
                    map_size.x / render_chunk_size + 1
                }
            },
            {
                if map_size.y % render_chunk_size == 0 {
                    map_size.y / render_chunk_size
                } else {
                    map_size.y / render_chunk_size + 1
                }
            },
        )
    }
}
