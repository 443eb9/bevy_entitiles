use bevy::{
    prelude::{IVec2, Mat4, Mesh, UVec2, Vec2},
    render::{
        mesh::{GpuBufferInfo, GpuMesh},
        render_resource::{BufferInitDescriptor, BufferUsages, PrimitiveTopology},
        renderer::RenderDevice,
    },
};

use super::{TileTexture, TileType};

pub struct TileChunk {
    pub dirty_mesh: bool,
    pub tile_type: TileType,
    pub coordinate: IVec2,
    pub size: UVec2,
    pub texture: TileTexture,
    pub tiles: Vec<TileData>,
    pub transform: Mat4,
    pub mesh: Mesh,
    pub gpu_mesh: GpuMesh,
}

pub struct TileData {
    pub coordinate: UVec2,
    pub texture_index: u32,
    position: Vec2,
}

impl TileChunk {
    pub fn set(&mut self, coordinate: IVec2, data: TileData) {
        self.dirty_mesh = true;
        self.tiles[coordinate.y as usize * self.size.x as usize + coordinate.x as usize] = data;
    }

    pub fn update_mesh(&mut self, device: &RenderDevice) {
        if !self.dirty_mesh {
            return;
        }

        for tile in self.tiles.iter() {}

        let vertex_buffer = device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("tilemap_vertex_buffer"),
            contents: &self.mesh.get_vertex_buffer_data(),
            usage: BufferUsages::VERTEX,
        });

        self.dirty_mesh = false;
        self.gpu_mesh = GpuMesh {
            vertex_buffer,
            vertex_count,
            morph_targets: None,
            buffer_info,
            primitive_topology: PrimitiveTopology::TriangleList,
            layout: None,
        }
    }
}
