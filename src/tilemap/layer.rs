use bevy::{ecs::{
    component::Component,
    entity::Entity,
    system::{ParallelCommands, Query},
}, reflect::Reflect, math::Vec4};

use super::tile::Tile;

#[derive(Component, Reflect)]
pub struct LayerInserter {
    pub is_top: bool,
    pub is_overwrite_if_full: bool,
    pub texture_index: u32,
    pub flip: Option<u32>,
}

pub fn layer_inserter(
    commands: ParallelCommands,
    mut tiles_query: Query<(Entity, &mut Tile, &LayerInserter)>,
) {
    tiles_query
        .par_iter_mut()
        .for_each(|(entity, mut tile, inserter)| {
            if inserter.is_top {
                tile.texture_indices.push(inserter.texture_index as i32);
            } else {
                tile.texture_indices.insert(0, inserter.texture_index as i32);
            }

            commands.command_scope(|mut c| {
                c.entity(entity).remove::<LayerInserter>();
            });
        });
}

#[derive(Default, Component, Clone, Copy, Reflect)]
pub struct LayerUpdater {
    pub texture_index: Option<(usize, u32)>,
    pub color: Option<Vec4>,
    pub flip: Option<(usize, u32)>,
}

pub fn layer_updater(mut tiles_query: Query<(&mut Tile, &LayerUpdater)>) {
    tiles_query.par_iter_mut().for_each(|(mut tile, updater)| {
        if let Some((layer, texture_index)) = updater.texture_index {
            if layer >= tile.texture_indices.len() {
                tile.texture_indices.resize(layer + 1, -1);
            }
            tile.texture_indices[layer] = texture_index as i32;
        }
        if let Some(color) = updater.color {
            tile.color = color;
        }
        if let Some((layer, flip)) = updater.flip {
            tile.flip[layer] = flip;
        }
    });
}
