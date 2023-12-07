use bevy::ecs::{
    component::Component,
    entity::Entity,
    system::{ParallelCommands, Query},
};

use super::tile::{AnimatedTile, Tile};

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
        .for_each(|(entity, mut tile, mut animated_tile, layer)| {
            if layer.is_remove && animated_tile.is_none() {
                remove_layer(&mut tile, layer);
            } else {
                update_layer(&mut tile, animated_tile.as_deref_mut(), layer);
            }

            commands.command_scope(|mut c| {
                c.entity(entity).remove::<UpdateLayer>();
            });
        })
}

fn remove_layer(tile: &mut Tile, layer: &UpdateLayer) {
    let (mut top_layer, mut available_layers) = (tile.top_layer, 0);
    for i in 0..tile.texture_index.len() {
        if tile.texture_index[i].is_some() {
            available_layers += 1;
        }
        if top_layer == tile.top_layer && i != layer.target {
            top_layer = i;
        }
    }
    if available_layers != 1 {
        tile.top_layer = top_layer;
        tile.texture_index[layer.target] = None;
    }
}

fn update_layer(tile: &mut Tile, animated_tile: Option<&mut AnimatedTile>, layer: &UpdateLayer) {
    if let Some(anim) = animated_tile {
        anim.layer = layer.target;
    } else if layer.target >= tile.texture_index.len() {
        let n = layer.target as i32 - tile.texture_index.len() as i32;
        if n > 0 {
            tile.texture_index.extend(vec![None; n as usize]);
        }

        tile.top_layer = layer.target;
        tile.texture_index.push(Some(layer.value));
    } else {
        tile.top_layer = tile.top_layer.max(layer.target);
        tile.texture_index[layer.target] = Some(layer.value);
    }
}
