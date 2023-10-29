use bevy::{
    prelude::{
        OrthographicProjection, Query, ResMut, UVec2, Component, Resource,
    },
    render::Extract
};

use crate::tilemap::Tilemap;

#[derive(Component)]
pub struct ExtractedView {
    pub projection: OrthographicProjection,
}

pub struct ExtractedTilemap {
    pub id: u32,
    pub size: UVec2,
}

#[derive(Resource, Default)]
pub struct ExtractedData {
    pub tilemaps: Vec<ExtractedTilemap>,
}

pub fn extract(
    tilemaps_query: Extract<Query<&Tilemap>>,
    mut extracted_data: ResMut<ExtractedData>,
) {
    extracted_data.tilemaps.clear();
    extracted_data.tilemaps.extend(tilemaps_query.iter().map(|tilemap| {
        ExtractedTilemap {
            id: tilemap.id,
            size: tilemap.size,
        }
    }));
}
