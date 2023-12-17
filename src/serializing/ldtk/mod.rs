use std::fs::read_to_string;

use bevy::{
    asset::AssetServer,
    ecs::{
        component::Component,
        entity::Entity,
        system::{Commands, NonSend, Query, Res},
    },
    math::{UVec2, Vec2},
    render::render_resource::FilterMode,
    sprite::{Sprite, SpriteBundle},
    transform::components::Transform,
    utils::hashbrown::HashMap,
};

use crate::render::texture::{TilemapTexture, TilemapTextureDescriptor};

use self::{
    entity::LdtkEntityRegistry,
    json::{
        definitions::{LayerType, TilesetDef},
        level::LayerInstance,
        LdtkJson, WorldLayout,
    },
    layer::LdtkLayers,
};

pub mod app_ext;
pub mod entity;
pub mod r#enum;
pub mod json;
pub mod layer;

#[derive(Component)]
pub struct LdtkLoader {
    /// The path to the ldtk file relative to the working directory.
    pub path: String,
    /// The path to the ldtk file relative to the assets folder.
    ///
    /// For example, your ldtk file is located at `assets/ldtk/fantastic_map.ldtk`,
    /// so this value will be `ldtk/`.
    pub asset_path_prefix: String,
    /// The level to load.
    pub level: Option<u32>,
    /// If you are using a map with `WorldLayout::LinearHorizontal` or `WorldLayout::LinearVertical` layout,
    /// and you are going to load all the levels,
    /// this value will be used to determine the spacing between the levels.
    pub level_spacing: Option<i32>,
    /// The `world_depth` of the [`Level`](crate::serializing::ldtk::json::level::Level).
    pub at_depth: i32,
    /// The filter mode of the tilemap texture.
    pub filter_mode: FilterMode,
    /// If `true`, then the entities with unregistered identifiers will be ignored.
    /// If `false`, then the program will panic.
    pub ignore_unregistered_entities: bool,
    /// Currently, multiple tilesets are not supported yet,
    /// so this value is to determine which tileset to use.
    ///
    /// If you only have one tileset, then you can leave this `None`.
    pub use_tileset: Option<usize>,
    /// The z index of the tilemap.
    pub z_index: i32,
}

pub fn load_ldtk_json(
    mut commands: Commands,
    loader_query: Query<(Entity, &LdtkLoader)>,
    asset_server: Res<AssetServer>,
    ident_mapper: NonSend<LdtkEntityRegistry>,
) {
    for (entity, loader) in loader_query.iter() {
        let path = std::env::current_dir().unwrap().join(&loader.path);
        let str_raw = match read_to_string(&path) {
            Ok(data) => data,
            Err(e) => panic!("Could not read file at path: {:?}!\n{}", path, e),
        };

        let mut ldtk_data = match serde_json::from_str::<LdtkJson>(&str_raw) {
            Ok(data) => data,
            Err(e) => panic!("Could not parse file at path: {}!\n{}", loader.path, e),
        };

        load_levels(
            &mut commands,
            &mut ldtk_data,
            loader,
            &asset_server,
            &ident_mapper,
        );
        commands.entity(entity).despawn();
    }
}

fn load_levels(
    commands: &mut Commands,
    ldtk_data: &mut LdtkJson,
    loader: &LdtkLoader,
    asset_server: &AssetServer,
    ident_mapper: &LdtkEntityRegistry,
) {
    let mut tilesets = HashMap::with_capacity(ldtk_data.defs.tilesets.len());
    ldtk_data.defs.tilesets.iter().for_each(|tileset| {
        if let Some(texture) = load_texture(tileset, &loader, asset_server) {
            tilesets.insert(tileset.uid, texture);
        }
    });

    for (level_index, level) in ldtk_data.levels.iter().enumerate() {
        if level.world_depth != loader.at_depth
            || loader.level.unwrap_or(level_index as u32) != level_index as u32
        {
            continue;
        }

        let translation = get_level_translation(&ldtk_data, loader, level_index);

        let level_px = UVec2 {
            x: level.px_wid as u32,
            y: level.px_hei as u32,
        };
        commands.spawn(SpriteBundle {
            sprite: Sprite {
                color: level.bg_color.into(),
                custom_size: Some(level_px.as_vec2()),
                ..Default::default()
            },
            transform: Transform::from_translation(
                (translation + level_px.as_vec2() / 2.)
                    .extend(loader.z_index as f32 - level.layer_instances.len() as f32 - 1.),
            ),
            ..Default::default()
        });

        let mut layer_grid = LdtkLayers::new(
            level.layer_instances.len(),
            level_px,
            &tilesets,
            translation,
            loader.z_index,
        );
        for (layer_index, layer) in level.layer_instances.iter().enumerate() {
            load_layer(
                commands,
                layer_index,
                layer,
                &mut layer_grid,
                &ident_mapper,
                loader,
                asset_server,
            );
        }

        layer_grid.apply_all(commands);
    }
}

fn load_texture(
    tileset: &TilesetDef,
    loader: &LdtkLoader,
    asset_server: &AssetServer,
) -> Option<TilemapTexture> {
    let Some(path) = tileset.rel_path.as_ref() else {
        return None;
    };

    let texture = asset_server.load(format!("{}{}", loader.asset_path_prefix, path));
    let desc = TilemapTextureDescriptor {
        size: UVec2 {
            x: tileset.px_wid as u32,
            y: tileset.px_hei as u32,
        },
        tile_size: UVec2 {
            x: tileset.tile_grid_size as u32,
            y: tileset.tile_grid_size as u32,
        },
        filter_mode: loader.filter_mode,
    };
    Some(TilemapTexture { texture, desc })
}

fn load_layer(
    commands: &mut Commands,
    layer_index: usize,
    layer: &LayerInstance,
    layer_grid: &mut LdtkLayers,
    ident_mapper: &LdtkEntityRegistry,
    loader: &LdtkLoader,
    asset_server: &AssetServer,
) {
    match layer.ty {
        LayerType::IntGrid | LayerType::AutoLayer => {
            for tile in layer.auto_layer_tiles.iter() {
                layer_grid.set(commands, layer_index, layer, tile);
            }
        }
        LayerType::Entities => {
            for entity in layer.entity_instances.iter() {
                let marker = {
                    if let Some(m) = ident_mapper.get(&entity.identifier) {
                        m
                    } else if !loader.ignore_unregistered_entities {
                        panic!(
                            "Could not find entity type with entity identifier: {}! \
                            You need to register it using App::register_ldtk_entity() first!",
                            entity.identifier
                        );
                    } else {
                        return;
                    }
                };

                let mut new_entity = commands.spawn_empty();
                marker.spawn(&mut new_entity, None, entity, asset_server);
            }
        }
        LayerType::Tiles => {
            for tile in layer.grid_tiles.iter() {
                layer_grid.set(commands, layer_index, layer, tile);
            }
        }
    }
}

fn get_level_translation(ldtk_data: &LdtkJson, loader: &LdtkLoader, index: usize) -> Vec2 {
    // TODO change this after LDtk update
    let level = &ldtk_data.levels[index];
    match ldtk_data.world_layout.unwrap() {
        WorldLayout::Free => todo!(),
        WorldLayout::GridVania => Vec2 {
            x: level.world_x as f32,
            y: (-level.world_y - level.px_hei) as f32,
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
                y: -offset as f32,
            }
        }
    }
}
