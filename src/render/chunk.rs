use bevy::{
    prelude::{Handle, Image, Mesh, UVec2, Vec3},
    render::{
        mesh::{GpuBufferInfo, GpuMesh, Indices},
        render_resource::{
            BufferInitDescriptor, BufferUsages, IndexFormat, PrimitiveTopology,
        },
        renderer::RenderDevice,
    },
};

use crate::tilemap::{
    Tile, Tilemap, TileType, TILEMAP_MESH_ATTR_POSITION, TILEMAP_MESH_ATTR_TEXTURE_INDEX,
};

#[derive(Clone)]
pub struct TileData {
    pub coordinate: UVec2,
    pub world_pos: Vec3,
    pub texture_index: u32,
}

#[derive(Clone)]
pub struct TileRenderChunk {
    pub dirty_mesh: bool,
    pub tile_type: TileType,
    pub size: UVec2,
    pub texture: Handle<Image>,
    pub tiles: Vec<TileData>,
    pub mesh: Mesh,
    pub gpu_mesh: Option<GpuMesh>,
}

impl TileRenderChunk {
    pub fn new(tilemap: &Tilemap, size: UVec2, tiles: Option<Vec<TileData>>) -> Self {
        TileRenderChunk {
            size,
            tile_type: tilemap.tile_type.clone(),
            texture: tilemap.texture.clone(),
            tiles: tiles.unwrap_or(vec![]),
            mesh: Mesh::new(PrimitiveTopology::TriangleList),
            gpu_mesh: None,
            dirty_mesh: true,
        }
    }

    /// Update the raw mesh for GPU processing.
    pub fn update_mesh(&mut self, device: &RenderDevice) {
        if !self.dirty_mesh {
            return;
        }

        let mut v_index = 0;
        let len = self.tiles.len();
        let mut positions = Vec::with_capacity(len * 4);
        let mut texture_indices = Vec::with_capacity(len);
        let mut indices = Vec::with_capacity(len * 6);

        for tile in self.tiles.iter() {
            positions.extend_from_slice(&[
                tile.world_pos,
                tile.world_pos,
                tile.world_pos,
                tile.world_pos,
            ]);

            texture_indices.push(tile.texture_index);

            indices.extend_from_slice(&[
                v_index,
                v_index + 2,
                v_index + 1,
                v_index,
                v_index + 3,
                v_index + 2,
            ]);

            v_index += 4;
        }

        self.mesh
            .insert_attribute(TILEMAP_MESH_ATTR_POSITION, positions);
        self.mesh
            .insert_attribute(TILEMAP_MESH_ATTR_TEXTURE_INDEX, texture_indices);
        self.mesh.set_indices(Some(Indices::U32(indices)));

        let mesh_vert_cout = self.mesh.count_vertices() as u32;

        let vertex_buffer = device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("tilemap_vertex_buffer"),
            contents: &self.mesh.get_vertex_buffer_data(),
            usage: BufferUsages::VERTEX,
        });

        let buffer_info =
            self.mesh
                .get_index_buffer_bytes()
                .map_or(GpuBufferInfo::NonIndexed, |data| GpuBufferInfo::Indexed {
                    buffer: device.create_buffer_with_data(&BufferInitDescriptor {
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

    /// Update the chunk that the tile is in.
    ///
    /// Or create a new chunk if it doesn't exist.
    pub(crate) fn update_tiles(&mut self, tile: Tile) {
        self.tiles.push(TileData {
            coordinate: tile.coordinate,
            world_pos: tile.world_pos,
            texture_index: tile.texture_index,
        });
        self.dirty_mesh = true;
    }
}
