use bevy::{
    prelude::{Camera, Commands, Entity, OrthographicProjection, Query, ResMut},
    render::Extract,
};

use crate::tilemap::Tile;

use super::{ExtractedData, ExtractedTile, ExtractedView};

pub fn extract(
    mut commands: Commands,
    camera: Extract<Query<(Entity, &Camera, &OrthographicProjection)>>,
    tiles: Extract<Query<&Tile>>,
    mut extracted_tiles: ResMut<ExtractedData>,
) {
    for (entity, camera, projection) in camera.iter() {
        commands.entity(entity).insert(ExtractedView {
            projection: projection.clone(),
        });
    }

    extracted_tiles.tiles = vec![];

    for tile in tiles.iter() {
        extracted_tiles.tiles.push(ExtractedTile {
            texture_index: tile.texture_index,
            coordinate: tile.coordinate,
        });
    }
}
