use bevy::{
    ecs::system::Commands,
    math::{IVec2, UVec2, Vec2, Vec4},
    utils::HashMap,
};

use crate::{
    render::texture::TilemapTexture,
    tilemap::{
        map::{Tilemap, TilemapBuilder},
        tile::{TileBuilder, TileType},
    },
};

use super::json::level::{LayerInstance, TileInstance};

pub struct LdtkLayers<'a> {
    pub layers: Vec<Option<Tilemap>>,
    pub tilesets: &'a HashMap<i32, TilemapTexture>,
    pub px_size: UVec2,
    pub translation: Vec2,
    pub base_z_index: i32,
}

impl<'a> LdtkLayers<'a> {
    pub fn new(
        total_layers: usize,
        px_size: UVec2,
        tilesets: &'a HashMap<i32, TilemapTexture>,
        translation: Vec2,
        base_z_index: i32,
    ) -> Self {
        Self {
            layers: vec![None; total_layers],
            tilesets,
            px_size,
            translation,
            base_z_index,
        }
    }

    pub fn set(
        &mut self,
        commands: &mut Commands,
        layer_index: usize,
        layer: &LayerInstance,
        tile: &TileInstance,
    ) {
        let tilemap = match self.layers[layer_index] {
            Some(ref mut tilemap) => tilemap,
            None => self.create_new_layer(commands, layer_index, layer),
        };

        let tile_size = tilemap.texture.as_ref().unwrap().desc.tile_size;
        let tile_index = IVec2 {
            x: tile.px[0] / tile_size.x as i32,
            y: tilemap.size.y as i32 - 1 - tile.px[1] / tile_size.y as i32,
        };
        if tilemap.is_out_of_tilemap_ivec(tile_index) {
            return;
        }

        let tile_index = tile_index.as_uvec2();

        if tilemap.get(tile_index).is_none() {
            let mut builder = TileBuilder::new()
                .with_layer(0, tile.tile_id as u32)
                .with_color(Vec4::new(1., 1., 1., tile.alpha));
            builder.flip[0] = tile.flip as u32;

            tilemap.set(commands, tile_index, &builder);
            (0..4).into_iter().for_each(|i| {
                tilemap.set_layer_opacity(i, layer.opacity);
            });
        } else {
            tilemap.insert_layer(
                commands,
                tile_index,
                tile.tile_id as u32,
                Some((tile.flip as u32).into()),
                true,
                false,
            );
        }
    }

    fn create_new_layer(
        &mut self,
        commands: &mut Commands,
        layer_index: usize,
        layer: &LayerInstance,
    ) -> &mut Tilemap {
        let tileset = self
            .tilesets
            .get(&layer.tileset_def_uid.unwrap())
            .cloned()
            .unwrap();

        let tilemap = TilemapBuilder::new(
            TileType::Square,
            UVec2 {
                x: layer.c_wid as u32,
                y: layer.c_hei as u32,
            },
            tileset.desc.tile_size.as_vec2(),
            layer.identifier.clone(),
        )
        .with_texture(tileset)
        .with_translation(self.translation)
        .with_z_index(self.base_z_index - layer_index as i32)
        .build(commands);

        self.layers[layer_index] = Some(tilemap);
        self.layers[layer_index].as_mut().unwrap()
    }

    pub fn apply_all(&mut self, commands: &mut Commands) {
        for tilemap in self.layers.drain(..) {
            if let Some(tm) = tilemap {
                commands.entity(tm.id).insert(tm);
            }
        }
    }
}
