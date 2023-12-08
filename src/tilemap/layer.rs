use bevy::ecs::{
    component::Component,
    entity::Entity,
    system::{ParallelCommands, Query},
};

use super::tile::{AnimatedTile, Tile, TileBuilder};

#[derive(Component)]
pub struct UpdateLayer {
    pub target: usize,
    pub value: u32,
    pub is_remove: bool,
}

pub fn layer_updater(
    commands: ParallelCommands,
    mut tile_query: Query<(Entity, &mut Tile, Option<&mut AnimatedTile>, &UpdateLayer)>,
) {
    tile_query
        .par_iter_mut()
        .for_each(|(entity, mut tile, mut animated_tile, updater)| {
            if updater.is_remove && animated_tile.is_none() {
                remove_layer(&mut tile, updater.target);
            } else {
                update_tile_layer(
                    &mut tile,
                    animated_tile.as_deref_mut(),
                    updater.target,
                    updater.value,
                );
            }

            commands.command_scope(|mut c| {
                c.entity(entity).remove::<UpdateLayer>();
            });
        })
}

pub fn remove_layer(tile: &mut Tile, layer: usize) {
    let (mut top_layer, mut available_layers) = (tile.top_layer, 0);
    for i in 0..tile.texture_index.len() {
        if tile.texture_index[i].is_some() {
            available_layers += 1;
        }
        if top_layer == tile.top_layer && i != layer {
            top_layer = i;
        }
    }
    if available_layers != 1 {
        tile.top_layer = top_layer;
        tile.texture_index[layer] = None;
    }
}

pub fn update_tile_layer(
    tile: &mut Tile,
    animated_tile: Option<&mut AnimatedTile>,
    layer: usize,
    texture_index: u32,
) {
    if let Some(anim) = animated_tile {
        anim.layer = layer;
    } else if layer >= tile.texture_index.len() {
        let n = layer as i32 - tile.texture_index.len() as i32;
        if n > 0 {
            tile.texture_index.extend(vec![None; n as usize]);
        }

        tile.top_layer = layer;
        tile.texture_index.push(Some(texture_index));
    } else {
        tile.top_layer = tile.top_layer.max(layer);
        tile.texture_index[layer] = Some(texture_index);
    }
}

pub fn update_tile_builder_layer(tile: &mut TileBuilder, layer: usize, texture_index: u32) {
    if let Some(anim) = tile.anim.as_mut() {
        anim.layer = layer;
    } else if layer >= tile.texture_index.len() {
        let n = layer as i32 - tile.texture_index.len() as i32;
        if n > 0 {
            tile.texture_index.extend(vec![None; n as usize]);
        }

        tile.top_layer = layer;
        tile.texture_index.push(Some(texture_index));
    } else {
        tile.top_layer = tile.top_layer.max(layer);
        tile.texture_index[layer] = Some(texture_index);
    }
}
