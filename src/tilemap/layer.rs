use bevy::{ecs::{
    component::Component,
    entity::Entity,
    system::{ParallelCommands, Query},
}, reflect::Reflect};

use crate::MAX_LAYER_COUNT;

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
                insert_top(&mut tile, inserter);
            } else {
                insert_bottom(&mut tile, inserter);
            }

            commands.command_scope(|mut c| {
                c.entity(entity).remove::<LayerInserter>();
            });
        });
}

fn insert_top(tile: &mut Tile, inserter: &LayerInserter) {
    let mut j = MAX_LAYER_COUNT;
    for i in (0..MAX_LAYER_COUNT).rev() {
        if tile.texture_indices[i] > 0 {
            if j < MAX_LAYER_COUNT {
                tile.texture_indices[j] = inserter.texture_index as i32;
                if let Some(f) = inserter.flip {
                    tile.flip[j] = f;
                }
                return;
            }
            break;
        }
        j -= 1;
    }
    if inserter.is_overwrite_if_full {
        tile.texture_indices[MAX_LAYER_COUNT - 1] = inserter.texture_index as i32;
    }
}

fn insert_bottom(tile: &mut Tile, inserter: &LayerInserter) {
    let mut j = -1;
    for i in 0..MAX_LAYER_COUNT {
        if tile.texture_indices[i] > 0 {
            if j > 0 {
                tile.texture_indices[j as usize] = inserter.texture_index as i32;
                if let Some(f) = inserter.flip {
                    tile.flip[j as usize] = f;
                }
                return;
            }
            break;
        }
        j += 1;
    }
    if inserter.is_overwrite_if_full {
        tile.texture_indices[0] = inserter.texture_index as i32;
    }
}
