use bevy::{
    ecs::{component::Component, entity::EntityHashMap, event::Event},
    math::{IVec2, IVec4, UVec4},
    prelude::{Entity, Mesh, Resource, Vec3, Vec4},
    reflect::Reflect,
    render::{
        mesh::{GpuBufferInfo, GpuMesh, Indices},
        render_asset::RenderAssetUsages,
        render_resource::{BufferInitDescriptor, BufferUsages, IndexFormat, PrimitiveTopology},
        renderer::RenderDevice,
    },
    utils::HashMap,
};

use crate::{
    math::{aabb::Aabb2d, extension::DivToFloor},
    tilemap::{
        map::{TilemapTexture, TilemapType},
        tile::TileTexture,
    },
    MAX_LAYER_COUNT,
};

use super::{
    extract::{ExtractedTile, ExtractedTilemap},
    material::{StandardTilemapMaterial, TilemapMaterialInstances},
    TILEMAP_MESH_ATTR_COLOR, TILEMAP_MESH_ATTR_FLIP, TILEMAP_MESH_ATTR_INDEX,
    TILEMAP_MESH_ATTR_TEX_INDICES,
};

#[derive(Component, Default, Debug, Clone, Reflect)]
pub struct UnloadRenderChunk(pub Vec<IVec2>);

#[derive(Event, Debug, Clone)]
pub struct ChunkUnload {
    pub tilemap: Entity,
    pub index: IVec2,
}

#[derive(Clone)]
pub struct MeshTileData {
    // When the third and forth component of index are not -1,
    // it means this tile is a animated tile
    // So the zw components are the start index and the length of the animation sequence
    pub index: IVec4,
    // 4 layers
    pub texture_indices: IVec4,
    pub tint: Vec4,
    pub flip: UVec4,
}

#[derive(Clone)]
pub struct TilemapRenderChunk {
    pub visible: bool,
    pub index: IVec2,
    pub dirty_mesh: bool,
    pub ty: TilemapType,
    pub size: u32,
    pub texture: Option<TilemapTexture>,
    pub tiles: Vec<Option<MeshTileData>>,
    pub mesh: Mesh,
    pub gpu_mesh: Option<GpuMesh>,
    pub aabb: Aabb2d,
}

impl TilemapRenderChunk {
    pub fn from_index(
        index: IVec2,
        tilemap: &ExtractedTilemap,
        materials: &TilemapMaterialInstances<StandardTilemapMaterial>,
    ) -> Self {
        TilemapRenderChunk {
            visible: true,
            index: index.div_to_floor(IVec2::splat(tilemap.chunk_size as i32)),
            size: tilemap.chunk_size,
            ty: tilemap.ty,
            texture: materials
                .get(&tilemap.material)
                .and_then(|m| m.texture.clone()),
            tiles: vec![None; (tilemap.chunk_size * tilemap.chunk_size) as usize],
            mesh: Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::RENDER_WORLD,
            ),
            gpu_mesh: None,
            dirty_mesh: true,
            aabb: Aabb2d::from_tilemap(
                index,
                tilemap.chunk_size,
                tilemap.ty,
                tilemap.tile_pivot,
                tilemap.axis_flip,
                tilemap.slot_size,
                tilemap.transform,
            ),
        }
    }

    /// Update the raw mesh for GPU processing.
    pub fn try_update_mesh(&mut self, render_device: &RenderDevice) {
        if !self.dirty_mesh {
            return;
        }
        let is_pure_color = self.texture.is_none();

        let mut v_index = 0;
        let len = self.tiles.len();

        let mut positions = Vec::with_capacity(len * 4);
        let mut texture_indices = Vec::with_capacity(len * 4);
        let mut grid_indices = Vec::with_capacity(len * 4);
        let mut vertex_indices = Vec::with_capacity(len * 6);
        let mut color = Vec::with_capacity(len * 4);
        let mut flip = Vec::with_capacity(len * 4);

        for tile_data in self.tiles.iter() {
            if let Some(tile) = tile_data {
                if !is_pure_color {
                    texture_indices.extend_from_slice(&[
                        tile.texture_indices,
                        tile.texture_indices,
                        tile.texture_indices,
                        tile.texture_indices,
                    ]);
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

                grid_indices.extend_from_slice(&[tile.index, tile.index, tile.index, tile.index]);
                color.extend_from_slice(&[tile.tint, tile.tint, tile.tint, tile.tint]);
                flip.extend_from_slice(&[tile.flip, tile.flip, tile.flip, tile.flip]);
            }
        }

        self.mesh
            .insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        self.mesh
            .insert_attribute(TILEMAP_MESH_ATTR_INDEX, grid_indices);
        self.mesh.insert_attribute(TILEMAP_MESH_ATTR_COLOR, color);
        if !is_pure_color {
            self.mesh
                .insert_attribute(TILEMAP_MESH_ATTR_TEX_INDICES, texture_indices);
            self.mesh.insert_attribute(TILEMAP_MESH_ATTR_FLIP, flip)
        }
        self.mesh.insert_indices(Indices::U32(vertex_indices));

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
    pub fn set_tile(&mut self, index: usize, tile: Option<&ExtractedTile>) {
        // TODO fix this. This allows the tile sort by y axis. But this approach looks weird.
        let index = self.tiles.len() - index - 1;

        let Some(tile) = tile else {
            self.tiles[index] = None;
            self.dirty_mesh = true;
            return;
        };

        let mut texture_indices = IVec4::NEG_ONE;
        let mut flip = UVec4::ZERO;
        let tile_index = {
            match &tile.texture {
                TileTexture::Static(tex) => {
                    tex.iter()
                        .rev()
                        .take(MAX_LAYER_COUNT)
                        .enumerate()
                        .for_each(|(i, t)| {
                            texture_indices[i] = t.texture_index;
                            flip[i] = t.flip.bits();
                        });
                    IVec4::new(tile.index.x, tile.index.y, -1, -1)
                }
                TileTexture::Animated(anim) => IVec4::new(
                    tile.index.x,
                    tile.index.y,
                    anim.start as i32,
                    anim.length as i32,
                ),
            }
        };

        self.tiles[index] = Some(MeshTileData {
            index: tile_index,
            texture_indices,
            tint: tile.tint.rgba_linear_to_vec4(),
            flip,
        });
        self.dirty_mesh = true;
    }
}

#[derive(Resource)]
pub struct RenderChunkStorage {
    pub(crate) value: EntityHashMap<HashMap<IVec2, TilemapRenderChunk>>,
}

impl Default for RenderChunkStorage {
    fn default() -> Self {
        Self {
            value: Default::default(),
        }
    }
}

impl RenderChunkStorage {
    /// Update the mesh for all chunks of a tilemap.
    pub fn prepare_chunks(&mut self, tilemap: &ExtractedTilemap, render_device: &RenderDevice) {
        if let Some(chunks) = self.value.get_mut(&tilemap.id) {
            chunks
                .values_mut()
                .for_each(|c| c.try_update_mesh(render_device));
        }
    }

    #[inline]
    pub fn get_chunks(&self, tilemap: Entity) -> Option<&HashMap<IVec2, TilemapRenderChunk>> {
        self.value.get(&tilemap)
    }

    #[inline]
    pub fn get_chunks_mut(
        &mut self,
        tilemap: Entity,
    ) -> Option<&mut HashMap<IVec2, TilemapRenderChunk>> {
        self.value.get_mut(&tilemap)
    }

    #[inline]
    pub fn remove_tilemap(
        &mut self,
        tilemap: Entity,
    ) -> Option<HashMap<IVec2, TilemapRenderChunk>> {
        self.value.remove(&tilemap)
    }

    #[inline]
    pub fn remove_chunk(&mut self, tilemap: Entity, index: IVec2) -> Option<TilemapRenderChunk> {
        self.value.get_mut(&tilemap).and_then(|c| c.remove(&index))
    }
}
