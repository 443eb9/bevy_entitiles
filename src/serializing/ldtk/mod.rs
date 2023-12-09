use std::fs::read_to_string;

use bevy::{
    asset::AssetServer,
    ecs::{
        component::Component,
        entity::Entity,
        system::{Commands, ParallelCommands, Query, Res},
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
    json::{LdtkJson, WorldLayout},
    level::{LayerInstance, Level},
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
    pub level_spacing: Option<i32>,
    pub at_depth: i32,
    pub scale: f32,
    pub filter_mode: FilterMode,
    pub z_order: i32,
}

pub fn load_ldtk_json(
    commands: ParallelCommands,
    mut loader_query: Query<(Entity, &LdtkLoader)>,
    asset_server: Res<AssetServer>,
) {
    loader_query.par_iter_mut().for_each(|(entity, loader)| {
        commands.command_scope(|mut cmd| {
            let Ok(str_raw) = read_to_string(&loader.path) else {
                panic!("Could not read file at path: {}!", loader.path);
            };

            let mut ldtk_data = match serde_json::from_str::<LdtkJson>(&str_raw) {
                Ok(data) => data,
                Err(e) => panic!("Could not parse file at path: {}!\n{}", loader.path, e),
            };

            load_ldtk(&mut ldtk_data, loader, &asset_server, &mut cmd);

            cmd.entity(entity).despawn();
        });
    });
}

fn load_ldtk(ldtk_data: &mut LdtkJson, loader: &LdtkLoader, asset_server: &Res<AssetServer>, commands: &mut Commands) {
    // texture
    // assert_eq!(
    //     ldtk_data.defs.tilesets.len(),
    //     1,
    //     "Multiple tilesets are not supported yet"
    // );
    let tileset = &ldtk_data.defs.tilesets[0];
    let texture = load_texture(tileset, &loader, &asset_server);

    // level
    for (level_index, level) in ldtk_data
        .levels
        .iter()
        .filter(|l| l.world_depth == loader.at_depth)
        .enumerate()
    {
        let translation = get_level_translation(&ldtk_data, loader, level_index);

        // background map
        let level_grid_size = UVec2 {
            x: (level.px_wid / tileset.tile_grid_size) as u32,
            y: (level.px_hei / tileset.tile_grid_size) as u32,
        };
        let (tilemap_entity, mut tilemap) = TilemapBuilder::new(
            TileType::Square,
            level_grid_size,
            tileset.tile_grid_size as f32 * loader.scale * Vec2::ONE,
            loader.tilemap_name.clone(),
        )
        .with_translation(translation)
        .build(commands);
        tilemap.fill_rect(
            commands,
            FillArea::full(&tilemap),
            &TileBuilder::new(0).with_color(level.bg_color.into()),
        );
        commands.entity(tilemap_entity).insert(tilemap);

        let mut layer_grid = Layer::new(level_grid_size);
        for (i, layer) in level.layer_instances.iter().enumerate() {
            load_layer(i, layer, &mut layer_grid);
        }

        let (tilemap_entity, mut tilemap) = TilemapBuilder::new(
            TileType::Square,
            level_grid_size,
            tileset.tile_grid_size as f32 * loader.scale * Vec2::ONE,
            loader.tilemap_name.clone(),
        )
        .with_translation(translation)
        .with_texture(texture.clone())
        .with_z_order(loader.z_order)
        .build(commands);

        tilemap.set_all(commands, &layer_grid.tiles);

        commands.entity(tilemap_entity).insert(tilemap);
    }
}

fn load_texture(tileset: &TilesetDef, loader: &LdtkLoader, asset_server: &Res<AssetServer>) -> TilemapTexture {
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
    // TODO support negative indices
    match layer.ty {
        LayerType::IntGrid | LayerType::AutoLayer => {
            for tile in layer.auto_layer_tiles.iter() {
                layer_grid.update(
                    UVec2 {
                        x: (tile.px[0] / layer.grid_size) as u32,
                        // the y axis is flipped in ldtk
                        y: (tile.px[1] / layer.grid_size) as u32,
                    },
                    layer_index,
                    tile.tile_id as u32,
                );
            }
        }
        LayerType::Entities => {
            // TODO
        }
        LayerType::Tiles => {
            for tile in layer.grid_tiles.iter() {
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
    }
}

fn get_level_translation(ldtk_data: &LdtkJson, loader: &LdtkLoader, index: usize) -> Vec2 {
    // TODO change this after LDtk update
    match ldtk_data.world_layout.unwrap() {
        WorldLayout::Free => todo!(),
        WorldLayout::GridVania => Vec2 {
            x: ldtk_data.levels[index].world_x as f32,
            y: ldtk_data.levels[index].world_y as f32,
        },
        WorldLayout::LinearHorizontal => {
            let mut offset = 0;
            for i in 0..index {
                offset += ldtk_data.levels[i].px_wid + loader.level_spacing.unwrap();
            }
            Vec2 {
                x: offset as f32,
                y: 0.,
            }
        }
        WorldLayout::LinearVertical => {
            let mut offset = 0;
            for i in 0..index {
                offset += ldtk_data.levels[i].px_hei + loader.level_spacing.unwrap();
            }
            Vec2 {
                x: 0.,
                y: offset as f32,
            }
        }
    }
}
