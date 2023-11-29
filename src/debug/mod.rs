use bevy::{
    ecs::entity::Entity,
    math::{UVec2, Vec2},
    render::render_resource::FilterMode,
};

use crate::{
    math::aabb::AabbBox2d,
    render::extract::ExtractedTilemap,
    tilemap::{
        map::Tilemap,
        tile::{TileType, TilemapTexture},
    },
};

pub struct PubTilemap {
    pub id: Entity,
    pub tile_type: TileType,
    pub size: UVec2,
    pub tile_render_scale: Vec2,
    pub tile_slot_size: Vec2,
    pub anchor: Vec2,
    pub render_chunk_size: u32,
    pub texture: Option<TilemapTexture>,
    pub filter_mode: FilterMode,
    pub tiles: Vec<Option<Entity>>,
    pub flip: u32,
    pub aabb: AabbBox2d,
    pub translation: Vec2,
    pub z_order: u32,
}

impl PubTilemap {
    pub fn from_tilemap(value: &Tilemap) -> Self {
        Self {
            id: value.id,
            tile_type: value.tile_type,
            size: value.size,
            tile_render_scale: value.tile_render_scale,
            tile_slot_size: value.tile_slot_size,
            anchor: value.anchor,
            render_chunk_size: value.render_chunk_size,
            texture: value.texture.clone(),
            filter_mode: value.filter_mode,
            tiles: value.tiles.clone(),
            flip: value.flip,
            aabb: value.aabb,
            translation: value.translation,
            z_order: value.z_order,
        }
    }

    pub fn from_extracted_tilemap(value: ExtractedTilemap) -> Self {
        Self {
            id: value.id,
            tile_type: value.tile_type,
            size: value.size,
            tile_render_scale: value.tile_render_scale,
            tile_slot_size: value.tile_slot_size,
            anchor: value.anchor,
            render_chunk_size: value.render_chunk_size,
            texture: value.texture,
            filter_mode: value.filter_mode,
            tiles: vec![],
            flip: value.flip,
            aabb: value.aabb,
            translation: value.translation,
            z_order: value.z_order,
        }
    }

    pub fn into_extracted_tilemap(self) -> ExtractedTilemap {
        ExtractedTilemap {
            id: self.id,
            tile_type: self.tile_type,
            size: self.size,
            tile_render_scale: self.tile_render_scale,
            tile_slot_size: self.tile_slot_size,
            anchor: self.anchor,
            render_chunk_size: self.render_chunk_size,
            texture: self.texture,
            filter_mode: self.filter_mode,
            translation: self.translation,
            flip: self.flip,
            aabb: self.aabb,
            z_order: self.z_order,
        }
    }
}
