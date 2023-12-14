use std::fs::read_to_string;

use bevy::{
    asset::AssetServer,
    ecs::{
        component::Component,
        entity::Entity,
        system::{Commands, NonSend, Query, Res},
    },
    math::{UVec2, Vec2, Vec4},
    render::render_resource::FilterMode,
    sprite::{Sprite, SpriteBundle},
    transform::components::Transform,
};

use crate::{
    render::texture::{TilemapTexture, TilemapTextureDescriptor},
    tilemap::{
        layer::update_tile_builder_layer,
        map::{Tilemap, TilemapBuilder},
        tile::{TileBuilder, TileType},
    },
    MAX_LAYER_COUNT,
};

use self::{
    entity::LdtkEntityIdentMapper,
    json::{
        definitions::{LayerType, TilesetDef},
        level::{LayerInstance, TileInstance},
        LdtkJson, WorldLayout,
    },
};

pub mod app_ext;
pub mod entity;
pub mod r#enum;
pub mod json;

#[derive(Component)]
pub struct LdtkLoader {
    /// The path to the ldtk file relative to the working directory.
    pub path: String,
    /// The path to the ldtk file relative to the assets folder.
    ///
    /// For example, your ldtk file is located at `assets/ldtk/fantastic_map.ldtk`,
    /// so this value will be `ldtk/`.
    pub asset_path_prefix: String,
    /// The name of the tilemap.
    /// If you are not going to save this tilemap, then fill this with dummy data.
    pub tilemap_name: String,
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
    /// The z order of the tilemap.
    pub z_order: i32,
}

pub fn load_ldtk_json(
    mut commands: Commands,
    loader_query: Query<(Entity, &LdtkLoader)>,
    asset_server: Res<AssetServer>,
    ident_mapper: NonSend<LdtkEntityIdentMapper>,
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

        load_ldtk(
            &mut commands,
            &mut ldtk_data,
            loader,
            &asset_server,
            &ident_mapper,
        );
        commands.entity(entity).despawn();
    }
}

fn load_ldtk(
    commands: &mut Commands,
    ldtk_data: &mut LdtkJson,
    loader: &LdtkLoader,
    asset_server: &AssetServer,
    ident_mapper: &LdtkEntityIdentMapper,
) {
    // texture
    let tileset_index = if let Some(idx) = loader.use_tileset {
        idx
    } else {
        // TODO remove this after multiple tilesets are supported
        assert_eq!(
            ldtk_data.defs.tilesets.len(),
            1,
            "Multiple tilesets are not supported yet!"
        );
        0
    };
    let tileset = &ldtk_data.defs.tilesets[tileset_index];
    let texture = load_texture(tileset, &loader, asset_server);

    assert!(
        ldtk_data
            .defs
            .layers
            .iter()
            .filter(|l| l.ty != LayerType::Entities)
            .count()
            <= MAX_LAYER_COUNT,
        "The maximum amount of rendered layers is {}!",
        MAX_LAYER_COUNT
    );

    // level
    for (level_index, level) in ldtk_data.levels.iter().enumerate() {
        if level.world_depth != loader.at_depth
            || loader.level.unwrap_or(level_index as u32) != level_index as u32
        {
            continue;
        }

        let translation = get_level_translation(&ldtk_data, loader, level_index);

        let level_grid_size = UVec2 {
            x: (level.px_wid / tileset.tile_grid_size) as u32,
            y: (level.px_hei / tileset.tile_grid_size) as u32,
        };
        let level_render_size = level_grid_size.as_vec2() * tileset.tile_grid_size as f32;

        commands.spawn(SpriteBundle {
            sprite: Sprite {
                color: level.bg_color.into(),
                custom_size: Some(level_render_size),
                ..Default::default()
            },
            transform: Transform::from_translation(
                (translation + level_render_size / 2.).extend(loader.z_order as f32 - 1.),
            ),
            ..Default::default()
        });

        let (tilemap_entity, mut tilemap) = TilemapBuilder::new(
            TileType::Square,
            level_grid_size,
            tileset.tile_grid_size as f32 * Vec2::ONE,
            loader.tilemap_name.clone(),
        )
        .with_translation(translation)
        .with_texture(texture.clone())
        .with_z_order(loader.z_order)
        .build(commands);

        let mut layer_grid = LdtkLayer::new(level_grid_size);
        let mut render_layer_index = MAX_LAYER_COUNT - 1;
        for layer in level.layer_instances.iter() {
            load_layer(
                commands,
                render_layer_index,
                layer,
                &mut layer_grid,
                &mut tilemap,
                &ident_mapper,
                loader,
                asset_server,
            );
            if layer.ty != LayerType::Entities {
                render_layer_index -= 1;
            }
        }

        tilemap.set_all(commands, &layer_grid.tiles);
        commands.entity(tilemap_entity).insert(tilemap);
    }
}

fn load_texture(
    tileset: &TilesetDef,
    loader: &LdtkLoader,
    asset_server: &AssetServer,
) -> TilemapTexture {
    let texture = asset_server.load(format!(
        "{}{}",
        loader.asset_path_prefix,
        tileset.rel_path.clone().unwrap()
    ));
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
    TilemapTexture { texture, desc }
}

pub struct LdtkLayer {
    pub tiles: Vec<Option<TileBuilder>>,
    pub size: UVec2,
}

impl LdtkLayer {
    pub fn new(size: UVec2) -> Self {
        Self {
            tiles: vec![None; (size.x * size.y) as usize],
            size,
        }
    }

    pub fn update(
        &mut self,
        layer_index: usize,
        layer_instance: &LayerInstance,
        tile_instance: &TileInstance,
        tilemap: &mut Tilemap,
    ) {
        if tile_instance.px[0] < 0 || tile_instance.px[1] < 0 {
            return;
        }

        let index = self.linear_index(UVec2 {
            x: (tile_instance.px[0] / layer_instance.grid_size) as u32,
            // the y axis is flipped in ldtk
            y: (layer_instance.c_hei - tile_instance.px[1] / layer_instance.grid_size - 1) as u32,
        });
        if index >= self.tiles.len() {
            return;
        }

        if let Some(tile) = self.tiles[index].as_mut() {
            update_tile_builder_layer(tile, layer_index, tile_instance.tile_id as u32);
            tilemap.set_layer_opacity(layer_index, layer_instance.opacity);
        } else {
            let mut builder = TileBuilder::new()
                .with_layer(layer_index, tile_instance.tile_id as u32)
                .with_color(Vec4::new(1., 1., 1., tile_instance.alpha));
            builder.flip = tile_instance.flip as u32;
            tilemap.set_layer_opacity(layer_index, layer_instance.opacity);
            self.tiles[index] = Some(builder);
        }
    }

    pub fn linear_index(&self, index: UVec2) -> usize {
        (index.y * self.size.x + index.x) as usize
    }
}

fn load_layer(
    commands: &mut Commands,
    layer_index: usize,
    layer: &LayerInstance,
    layer_grid: &mut LdtkLayer,
    tilemap: &mut Tilemap,
    ident_mapper: &LdtkEntityIdentMapper,
    loader: &LdtkLoader,
    asset_server: &AssetServer,
) {
    match layer.ty {
        LayerType::IntGrid | LayerType::AutoLayer => {
            for tile in layer.auto_layer_tiles.iter() {
                layer_grid.update(layer_index, layer, tile, tilemap);
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
                marker.spawn(&mut new_entity, entity, asset_server);
            }
        }
        LayerType::Tiles => {
            for tile in layer.grid_tiles.iter() {
                layer_grid.update(layer_index, layer, tile, tilemap);
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
