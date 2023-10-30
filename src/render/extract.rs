use bevy::{
    prelude::{
        Component, Entity, Handle, Image, OrthographicProjection, Query, ResMut, Resource, UVec2,
    },
    render::{render_resource::FilterMode, Extract},
};

use crate::tilemap::{TileType, Tilemap};

use super::texture::{TilemapTextureArrayStorage, TilemapTextureDescriptor};

#[derive(Component)]
pub struct ExtractedView {
    pub projection: OrthographicProjection,
}

pub struct ExtractedTilemap {
    pub entity: Entity,
    pub id: u32,
    pub size: UVec2,
    pub tile_size: UVec2,
    pub tile_type: TileType,
    pub filter_mode: FilterMode,
    pub texture: Handle<Image>,
    pub z_order: f32,
}

#[derive(Resource, Default)]
pub struct ExtractedData {
    pub tilemaps: Vec<ExtractedTilemap>,
}

pub fn extract(
    tilemaps_query: Extract<Query<(Entity, &Tilemap)>>,
    mut extracted_data: ResMut<ExtractedData>,
    mut tilemap_texture_array_storage: ResMut<TilemapTextureArrayStorage>,
) {
    extracted_data.tilemaps.clear();

    for (entity, tilemap) in tilemaps_query.iter() {
        extracted_data.tilemaps.push(ExtractedTilemap {
            entity,
            id: tilemap.id,
            size: tilemap.size,
            tile_size: tilemap.tile_size,
            tile_type: tilemap.tile_type.clone(),
            texture: tilemap.texture.clone_weak(),
            filter_mode: tilemap.filter_mode,
            z_order: tilemap.z_order,
        });
        tilemap_texture_array_storage.register(
            &tilemap.texture,
            TilemapTextureDescriptor {
                tile_size: tilemap.tile_size,
                tile_count: tilemap.size.length_squared(),
                filter_mode: tilemap.filter_mode,
            },
        )
    }
}
