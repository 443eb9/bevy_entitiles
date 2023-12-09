use std::fs::read_to_string;

use bevy::{
    asset::AssetServer,
    ecs::{
        component::Component,
        entity::Entity,
        system::{ParallelCommands, Query, Res},
    },
    math::{UVec2, Vec2},
    render::render_resource::FilterMode,
};

use crate::{
    math::FillArea,
    render::texture::{TilemapTexture, TilemapTextureDescriptor},
    serializing::ldtk::definitions::LayerType,
    tilemap::{
        layer::update_tile_builder_layer,
        map::TilemapBuilder,
        tile::{TileBuilder, TileType},
    },
};

use self::{
    definitions::TilesetDef,
    json::LdtkJson,
    level::LayerInstance,
};

pub mod definitions;
pub mod json;
pub mod level;
pub mod macros;

#[derive(Component)]
pub struct LdtkLoader {
    pub path: String,
    pub asset_path_prefix: String,
    pub tilemap_name: String,
    pub scale: f32,
    pub filter_mode: FilterMode,
    pub z_order: i32,
}

pub fn load_ldtk(
    commands: ParallelCommands,
    mut loader_query: Query<(Entity, &LdtkLoader)>,
    asset_server: Res<AssetServer>,
) {
    loader_query.par_iter_mut().for_each(|(entity, loader)| {
        commands.command_scope(|mut cmd| {
            let Ok(str_raw) = read_to_string(&loader.path) else {
                panic!("Could not read file at path: {}!", loader.path);
            };

            let ldtk_data = match serde_json::from_str::<LdtkJson>(&str_raw) {
                Ok(data) => data,
                Err(e) => panic!("Could not parse file at path: {}!\n{}", loader.path, e),
            };

            // texture
            assert_eq!(
                ldtk_data.defs.tilesets.len(),
                1,
                "Multiple tilesets are not supported yet"
            );
            let tileset = &ldtk_data.defs.tilesets[0];
            let texture = load_texture(tileset, &loader, &asset_server);

            // level
            let mut max_size = UVec2::ZERO;
            for level in ldtk_data.levels.iter() {
                // TODO multiple levels support
                for layer in level.layer_instances.iter() {
                    max_size.x = max_size.x.max(layer.c_wid as u32);
                    max_size.y = max_size.y.max(layer.c_hei as u32);
                }

                // background map
                let (tilemap_entity, mut tilemap) = TilemapBuilder::new(
                    TileType::Square,
                    max_size,
                    tileset.tile_grid_size as f32 * loader.scale * Vec2::ONE,
                    loader.tilemap_name.clone(),
                )
                .build(&mut cmd);
                tilemap.fill_rect(
                    &mut cmd,
                    FillArea::full(&tilemap),
                    &TileBuilder::new(0).with_color(level.bg_color.into()),
                );
                cmd.entity(tilemap_entity).insert(tilemap);

                let mut layer_grid = Layer::new(max_size);
                for (i, layer) in level.layer_instances.iter().enumerate() {
                    load_layer(i, layer, &mut layer_grid);
                }

                let (tilemap_entity, mut tilemap) = TilemapBuilder::new(
                    TileType::Square,
                    max_size,
                    tileset.tile_grid_size as f32 * loader.scale * Vec2::ONE,
                    loader.tilemap_name.clone(),
                )
                .with_texture(texture.clone())
                .with_z_order(loader.z_order)
                .build(&mut cmd);

                tilemap.set_all(&mut cmd, &layer_grid.tiles);

                cmd.entity(tilemap_entity).insert(tilemap);
            }

            cmd.entity(entity).despawn();
        });
    });
}

fn load_texture(
    tileset: &TilesetDef,
    loader: &LdtkLoader,
    asset_server: &Res<AssetServer>,
) -> TilemapTexture {
    let texture = asset_server.load(format!(
        "{}/{}",
        loader.asset_path_prefix,
        tileset.rel_path.clone().unwrap()
    ));
    let desc = TilemapTextureDescriptor::from_full_grid(
        UVec2 {
            x: tileset.px_wid as u32,
            y: tileset.px_hei as u32,
        },
        UVec2 {
            x: tileset.c_wid as u32,
            y: tileset.c_hei as u32,
        },
        loader.filter_mode,
    );
    TilemapTexture { texture, desc }
}

pub struct Layer {
    pub tiles: Vec<Option<TileBuilder>>,
    pub size: UVec2,
}

impl Layer {
    pub fn new(size: UVec2) -> Self {
        Self {
            tiles: vec![None; (size.x * size.y) as usize],
            size,
        }
    }

    pub fn update(&mut self, index: UVec2, layer: usize, texture_index: u32) {
        let index = self.linear_index(index);
        if index >= self.tiles.len() {
            return;
        }

        if let Some(tile) = self.tiles[index].as_mut() {
            update_tile_builder_layer(tile, layer, texture_index);
        } else {
            self.tiles[index] = Some(TileBuilder::new(texture_index));
        }
    }

    pub fn linear_index(&self, index: UVec2) -> usize {
        (index.y * self.size.x + index.x) as usize
    }
}

fn load_layer(layer_index: usize, layer: &LayerInstance, layer_grid: &mut Layer) {
    match layer.ty {
        LayerType::IntGrid | LayerType::AutoLayer => {
            for tile in layer.auto_layer_tiles.iter() {
                if tile.px[0] < 0 || tile.px[1] < 0 {
                    continue;
                }

                layer_grid.update(
                    UVec2 {
                        x: (tile.px[0] / layer.grid_size) as u32,
                        // the y axis is flipped in ldtk
                        y: (layer.c_hei - tile.px[1] / layer.grid_size) as u32,
                    },
                    layer_index,
                    tile.tile_id as u32,
                );
            }
        }
        LayerType::Entities => {
            todo!()
        }
        LayerType::Tiles => {
            todo!()
        }
    }
}
