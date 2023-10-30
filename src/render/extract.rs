use bevy::{
    prelude::{
        Commands, Component, Entity, Handle, Image, OrthographicProjection, Query, ResMut, UVec2,
    },
    render::{render_resource::FilterMode, Extract},
};

use crate::tilemap::{TileType, Tilemap};

use super::texture::{TilemapTextureArrayStorage, TilemapTextureDescriptor};

#[derive(Component)]
pub struct ExtractedView {
    pub projection: OrthographicProjection,
}

#[derive(Component)]
pub struct ExtractedTilemap {
    pub id: u32,
    pub size: UVec2,
    pub tile_size: UVec2,
    pub tile_type: TileType,
    pub texture: Handle<Image>,
    pub filter_mode: FilterMode,
    pub z_order: f32,
}

pub fn extract(
    mut commands: Commands,
    tilemaps_query: Extract<Query<(Entity, &Tilemap)>>,
    mut tilemap_texture_array_storage: ResMut<TilemapTextureArrayStorage>,
) {
    let mut extracted_tilemaps: Vec<(Entity, ExtractedTilemap)> = Vec::new();

    for (entity, tilemap) in tilemaps_query.iter() {
        extracted_tilemaps.push((
            entity,
            ExtractedTilemap {
                id: tilemap.id,
                size: tilemap.size,
                tile_size: tilemap.tile_size,
                tile_type: tilemap.tile_type.clone(),
                filter_mode: tilemap.filter_mode,
                texture: tilemap.texture.clone_weak(),
                z_order: tilemap.z_order,
            },
        ));

        tilemap_texture_array_storage.register(
            &tilemap.texture,
            TilemapTextureDescriptor {
                tile_size: tilemap.tile_size,
                tile_count: tilemap.size.length_squared(),
                filter_mode: tilemap.filter_mode,
            },
        )
    }

    commands.insert_or_spawn_batch(extracted_tilemaps);
}
