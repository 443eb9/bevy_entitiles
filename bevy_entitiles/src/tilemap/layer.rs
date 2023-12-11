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
    for i in 0..tile.texture_indices.len() {
        if tile.texture_indices[i] >= 0 {
            available_layers += 1;
        }
        if top_layer == tile.top_layer && i != layer {
            top_layer = i;
        }
    }
    if available_layers != 1 {
        tile.top_layer = top_layer;
        tile.texture_indices[layer] = -1;
    }
}

pub fn update_tile_layer(
    tile: &mut Tile,
    animated_tile: Option<&mut AnimatedTile>,
    layer: usize,
    texture_index: u32,
) {
    if animated_tile.is_none() {
        tile.top_layer = tile.top_layer.max(layer);
        tile.texture_indices[layer] = texture_index as i32;
    }
}

pub fn update_tile_builder_layer(tile: &mut TileBuilder, layer: usize, texture_index: u32) {
    if tile.anim.is_none() {
        tile.top_layer = tile.top_layer.max(layer);
        tile.texture_indices[layer] = texture_index as i32;
    }
}
