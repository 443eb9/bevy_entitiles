use std::{cmp::Ordering, marker::PhantomData};

use bevy::{
    asset::Handle, color::ColorToComponents, ecs::{component::Component, entity::EntityHashMap, event::Event}, math::{IVec2, IVec4}, prelude::{Entity, Mesh, Resource, Vec3, Vec4}, reflect::Reflect, render::{
        mesh::{BaseMeshPipelineKey, GpuBufferInfo, GpuMesh, Indices, MeshVertexBufferLayouts},
        render_asset::RenderAssetUsages,
        render_resource::{BufferInitDescriptor, BufferUsages, IndexFormat, PrimitiveTopology},
        renderer::RenderDevice,
    }
};
use indexmap::{map::Entry, IndexMap};
use rayon::iter::ParallelIterator;

use crate::{
    math::{aabb::Aabb2d, extension::DivToFloor},
    render::{
        extract::{ExtractedTile, ExtractedTilemap},
        material::TilemapMaterial,
        TILEMAP_MESH_ATTR_ATLAS_INDICES, TILEMAP_MESH_ATTR_COLOR, TILEMAP_MESH_ATTR_INDEX,
    },
    tilemap::{
        map::{TilemapTextures, TilemapType},
        tile::{Tile, TileTexture},
    },
    MAX_LAYER_COUNT,
};

#[cfg(feature = "atlas")]
use super::TILEMAP_MESH_ATTR_TEX_INDICES;

#[derive(Component, Default, Debug, Clone, Reflect)]
pub struct UnloadRenderChunk(pub Vec<IVec2>);

#[derive(Resource, Clone, Default)]
pub enum RenderChunkSort {
    #[default]
    None,
    XThenY,
    XReverseThenY,
    XThenYReverse,
    XReverseThenYReverse,
    YThenX,
    YReverseThenX,
    YThenXReverse,
    YReverseThenXReverse,
}

#[derive(Event, Debug, Clone)]
pub struct ChunkUnload {
    pub tilemap: Entity,
    pub index: IVec2,
}

#[derive(Debug, Clone)]
pub struct MeshTileData {
    // When the third and forth component of index are not -1,
    // it means this tile is a animated tile
    // So the zw components are the start index and the length of the animation sequence
    pub index: IVec4,
    // 4 layers
    #[cfg(feature = "atlas")]
    pub texture_indices: IVec4,
    pub atlas_indices: IVec4,
    pub tint: Vec4,
}

#[derive(Clone)]
pub struct TilemapRenderChunk<M: TilemapMaterial> {
    pub visible: bool,
    pub index: IVec2,
    pub dirty_mesh: bool,
    pub ty: TilemapType,
    pub size: u32,
    pub texture: Option<Handle<TilemapTextures>>,
    pub tiles: Vec<Option<MeshTileData>>,
    pub mesh: Mesh,
    pub gpu_mesh: Option<GpuMesh>,
    pub aabb: Aabb2d,
    pub marker: PhantomData<M>,
}

impl<M: TilemapMaterial> TilemapRenderChunk<M> {
    pub fn from_index(index: IVec2, tilemap: &ExtractedTilemap<M>) -> Self {
        TilemapRenderChunk {
            visible: true,
            index: index.div_to_floor(IVec2::splat(tilemap.chunk_size as i32)),
            size: tilemap.chunk_size,
            ty: tilemap.ty,
            texture: tilemap.texture.clone(),
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
            marker: PhantomData,
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
        #[cfg(feature = "atlas")]
        let mut texture_indices = Vec::with_capacity(len * 4);
        let mut atlas_indices = Vec::with_capacity(len * 4);
        let mut grid_indices = Vec::with_capacity(len * 4);
        let mut vertex_indices = Vec::with_capacity(len * 6);
        let mut color = Vec::with_capacity(len * 4);

        for tile_data in self.tiles.iter() {
            if let Some(tile) = tile_data {
                if !is_pure_color {
                    #[cfg(feature = "atlas")]
                    texture_indices.extend_from_slice(&[
                        tile.texture_indices,
                        tile.texture_indices,
                        tile.texture_indices,
                        tile.texture_indices,
                    ]);

                    atlas_indices.extend_from_slice(&[
                        tile.atlas_indices,
                        tile.atlas_indices,
                        tile.atlas_indices,
                        tile.atlas_indices,
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
            }
        }

        self.mesh
            .insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        self.mesh
            .insert_attribute(TILEMAP_MESH_ATTR_INDEX, grid_indices);
        self.mesh.insert_attribute(TILEMAP_MESH_ATTR_COLOR, color);
        if !is_pure_color {
            self.mesh
                .insert_attribute(TILEMAP_MESH_ATTR_ATLAS_INDICES, atlas_indices);
            #[cfg(feature = "atlas")]
            {
                self.mesh
                    .insert_attribute(TILEMAP_MESH_ATTR_TEX_INDICES, texture_indices);
            }
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
            layout: self
                .mesh
                .get_mesh_vertex_buffer_layout(&mut MeshVertexBufferLayouts::default()),
            key_bits: BaseMeshPipelineKey::from_primitive_topology(PrimitiveTopology::TriangleList),
        });

        self.dirty_mesh = false;
    }

    /// Set a tile in the chunk. Overwrites the previous tile.
    pub fn set_tile(&mut self, index: usize, tile: Option<&ExtractedTile>) {
        let Some(tile) = tile else {
            self.tiles[index] = None;
            self.dirty_mesh = true;
            return;
        };

        #[cfg(feature = "atlas")]
        let mut texture_indices = IVec4::NEG_ONE;
        let mut atlas_indices = IVec4::NEG_ONE;
        let tile_index = {
            match &tile.texture {
                TileTexture::Static(_) => IVec4::new(tile.index.x, tile.index.y, -1, -1),
                TileTexture::Animated(anim) => IVec4::new(
                    tile.index.x,
                    tile.index.y,
                    anim.start as i32,
                    anim.length as i32,
                ),
            }
        };

        if let TileTexture::Static(tex) = &tile.texture {
            tex.iter()
                .enumerate()
                .rev()
                .take(MAX_LAYER_COUNT)
                .for_each(|(i, t)| {
                    #[cfg(feature = "atlas")]
                    {
                        texture_indices[i] = t.texture_index;
                    }
                    let flip = t.flip.bits() as i32;
                    // Shift 29 bits but not 30 because it's a signed integer,
                    // and we need to identify if the layer is empty or not according to the sign.
                    atlas_indices[i] = t.atlas_index | (flip << 29);
                });
        }

        self.tiles[index] = Some(MeshTileData {
            index: tile_index,
            #[cfg(feature = "atlas")]
            texture_indices,
            atlas_indices,
            tint: tile.tint.to_vec4(),
        });
        self.dirty_mesh = true;
    }
}

#[derive(Resource)]
pub struct TilemapRenderChunks<M: TilemapMaterial> {
    pub tilemap: Entity,
    pub value: IndexMap<IVec2, TilemapRenderChunk<M>>,
    pub is_dirty: bool,
}

impl<M: TilemapMaterial> TilemapRenderChunks<M> {
    pub fn new(tilemap: Entity) -> Self {
        Self {
            tilemap,
            value: Default::default(),
            is_dirty: true,
        }
    }

    #[inline]
    pub fn try_add_chunk(&mut self, chunk_index: IVec2, tilemap: &ExtractedTilemap<M>) {
        match self.value.entry(chunk_index) {
            Entry::Occupied(_) => {}
            Entry::Vacant(e) => {
                e.insert(TilemapRenderChunk::from_index(chunk_index, tilemap));
                self.is_dirty = true;
            }
        }
    }

    #[inline]
    pub fn remove_chunk(&mut self, index: IVec2) -> Option<TilemapRenderChunk<M>> {
        self.value.shift_remove(&index)
    }

    #[inline]
    pub fn set_tile(&mut self, tile: &Tile) {
        if let Some(c) = self.value.get_mut(&tile.chunk_index) {
            c.set_tile(tile.in_chunk_index, Some(tile));
        }
    }

    #[inline]
    pub fn remove_tile(&mut self, chunk_index: IVec2, in_chunk_index: usize) {
        if let Some(c) = self.value.get_mut(&chunk_index) {
            c.set_tile(in_chunk_index, None);
        }
    }

    #[inline]
    pub fn try_sort(&mut self, f: impl Fn(IVec2, IVec2) -> Ordering + Send + Sync) {
        if self.is_dirty {
            self.value.par_sort_by(|lhs, _, rhs, _| f(*lhs, *rhs));
            self.is_dirty = false;
        }
    }
}

#[derive(Resource)]
pub struct RenderChunkStorage<M: TilemapMaterial> {
    pub(crate) value: EntityHashMap<TilemapRenderChunks<M>>,
}

impl<M: TilemapMaterial> Default for RenderChunkStorage<M> {
    fn default() -> Self {
        Self {
            value: Default::default(),
        }
    }
}

impl<M: TilemapMaterial> RenderChunkStorage<M> {
    /// Update the mesh for all chunks of a tilemap.
    pub fn prepare_chunks(&mut self, tilemap: &ExtractedTilemap<M>, render_device: &RenderDevice) {
        if let Some(chunks) = self.value.get_mut(&tilemap.id) {
            chunks
                .value
                .values_mut()
                .for_each(|c| c.try_update_mesh(render_device));
        }
    }

    #[inline]
    pub fn get_chunks(&self, tilemap: Entity) -> Option<&TilemapRenderChunks<M>> {
        self.value.get(&tilemap)
    }

    #[inline]
    pub fn get_or_insert_chunks(&mut self, tilemap: Entity) -> &mut TilemapRenderChunks<M> {
        self.value
            .entry(tilemap)
            .or_insert_with(|| TilemapRenderChunks::new(tilemap))
    }

    #[inline]
    pub fn remove_tilemap(&mut self, tilemap: Entity) -> Option<TilemapRenderChunks<M>> {
        self.value.remove(&tilemap)
    }

    #[inline]
    pub fn sort(&mut self, f: impl Fn(IVec2, IVec2) -> Ordering + Sync + Send + Copy) {
        self.value.par_values_mut().for_each(|c| c.try_sort(f));
    }
}
