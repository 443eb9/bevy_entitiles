use std::fs::read_to_string;

use bevy::{
    asset::AssetServer,
    ecs::{
        component::Component,
        entity::Entity,
        reflect::AppTypeRegistry,
        system::{Commands, ParallelCommands, Query, Res},
    },
    math::{UVec2, Vec2, Vec4},
    reflect::{
        serde::{ReflectSerializer, UntypedReflectDeserializer},
        DynamicStruct, Reflect, StructInfo, TypeInfo,
    },
    render::render_resource::FilterMode,
};

use crate::{
    math::FillArea,
    render::texture::{TilemapTexture, TilemapTextureDescriptor},
    tilemap::{
        layer::update_tile_builder_layer,
        map::{Tilemap, TilemapBuilder},
        tile::{TileBuilder, TileType},
    },
    MAX_LAYER_COUNT,
};

use self::json::{
    definitions::{LayerType, TilesetDef},
    level::{FieldValue, LayerInstance, TileInstance},
    LdtkColor, LdtkJson, WorldLayout,
};

pub mod entity;
pub mod r#enum;
pub mod json;

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
    type_registry: Res<AppTypeRegistry>,
) {
    loader_query.par_iter_mut().for_each(|(entity, loader)| {
        let path = std::env::current_dir().unwrap().join(&loader.path);
        let str_raw = match read_to_string(&path) {
            Ok(data) => data,
            Err(e) => panic!("Could not read file at path: {:?}!\n{}", path, e),
        };

        let mut ldtk_data = match serde_json::from_str::<LdtkJson>(&str_raw) {
            Ok(data) => data,
            Err(e) => panic!("Could not parse file at path: {}!\n{}", loader.path, e),
        };

        commands.command_scope(|mut cmd| {
            load_ldtk(
                &mut ldtk_data,
                loader,
                &asset_server,
                &mut cmd,
                &type_registry,
            );
            cmd.entity(entity).despawn();
        });
    });
}

fn load_ldtk(
    ldtk_data: &mut LdtkJson,
    loader: &LdtkLoader,
    asset_server: &Res<AssetServer>,
    commands: &mut Commands,
    type_registry: &Res<AppTypeRegistry>,
) {
    // texture
    // assert_eq!(
    //     ldtk_data.defs.tilesets.len(),
    //     1,
    //     "Multiple tilesets are not supported yet"
    // );
    let tileset = &ldtk_data.defs.tilesets[0];
    let texture = load_texture(tileset, &loader, &asset_server);

    // level
    for (level_index, level) in ldtk_data.levels.iter().enumerate() {
        if level.world_depth != loader.at_depth {
            continue;
        }

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
            &TileBuilder::new().with_color(level.bg_color.into()),
        );
        commands.entity(tilemap_entity).insert(tilemap);

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

        let mut layer_grid = Layer::new(level_grid_size);
        let mut render_layer_index = MAX_LAYER_COUNT - 1;
        for layer in level.layer_instances.iter() {
            load_layer(
                render_layer_index,
                layer,
                &mut layer_grid,
                type_registry,
                &mut tilemap,
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
    asset_server: &Res<AssetServer>,
) -> TilemapTexture {
    let texture = asset_server.load(format!(
        "{}/{}",
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

    pub fn update(
        &mut self,
        layer_index: usize,
        ldtk_layer: &LayerInstance,
        ldtk_tile: &TileInstance,
        tilemap: &mut Tilemap,
    ) {
        let index = self.linear_index(UVec2 {
            x: (ldtk_tile.px[0] / ldtk_layer.grid_size) as u32,
            // the y axis is flipped in ldtk
            y: (ldtk_layer.c_hei - ldtk_tile.px[1] / ldtk_layer.grid_size - 1) as u32,
        });
        if index >= self.tiles.len() {
            return;
        }

        if let Some(tile) = self.tiles[index].as_mut() {
            update_tile_builder_layer(tile, layer_index, ldtk_tile.tile_id as u32);
        } else {
            let mut builder = TileBuilder::new()
                .with_layer(layer_index, ldtk_tile.tile_id as u32)
                .with_color(Vec4::new(1., 1., 1., ldtk_tile.alpha));
            builder.flip = ldtk_tile.flip as u32;
            tilemap.set_layer_opacity(layer_index, ldtk_layer.opacity);
            self.tiles[index] = Some(builder);
        }
    }

    pub fn linear_index(&self, index: UVec2) -> usize {
        (index.y * self.size.x + index.x) as usize
    }
}

fn load_layer(
    layer_index: usize,
    layer: &LayerInstance,
    layer_grid: &mut Layer,
    type_registry: &Res<AppTypeRegistry>,
    tilemap: &mut Tilemap,
) {
    println!("{}: {}", layer.identifier, layer_index);
    match layer.ty {
        LayerType::IntGrid | LayerType::AutoLayer => {
            for tile in layer.auto_layer_tiles.iter() {
                if tile.px[0] < 0 || tile.px[1] < 0 {
                    continue;
                }

                layer_grid.update(layer_index, layer, tile, tilemap);
            }
        }
        LayerType::Entities => {
            for entity in layer.entity_instances.iter() {
                let mut e = DynamicStruct::default();
                for field in entity.field_instances.iter() {
                    if let Some(value) = field.value.clone() {
                        match value {
                            FieldValue::Integer(v) => e.insert(&field.identifier, v),
                            FieldValue::Float(v) => e.insert(&field.identifier, v),
                            FieldValue::Bool(v) => e.insert(&field.identifier, v),
                            FieldValue::String(v) => e.insert(&field.identifier, v),
                            FieldValue::LocalEnum(v) => e.insert(&field.identifier, v),
                            FieldValue::ExternEnum(v) => e.insert(&field.identifier, v),
                            FieldValue::Color(v) => e.insert(&field.identifier, v),
                            FieldValue::Point(v) => e.insert(&field.identifier, v),
                            FieldValue::EntityRef(v) => e.insert(&field.identifier, v),
                            FieldValue::IntegerArray(v) => e.insert(&field.identifier, v),
                            FieldValue::FloatArray(v) => e.insert(&field.identifier, v),
                            FieldValue::BoolArray(v) => e.insert(&field.identifier, v),
                            FieldValue::StringArray(v) => e.insert(&field.identifier, v),
                            FieldValue::LocalEnumArray(v) => e.insert(&field.identifier, v),
                            FieldValue::ExternEnumArray(v) => e.insert(&field.identifier, v),
                            FieldValue::ColorArray(v) => e.insert(&field.identifier, v),
                            FieldValue::PointArray(v) => e.insert(&field.identifier, v),
                            FieldValue::EntityRefArray(v) => e.insert(&field.identifier, v),
                        }
                    }
                }

                // TODO Spawn entity
            }
        }
        LayerType::Tiles => {
            for tile in layer.grid_tiles.iter() {
                if tile.px[0] < 0 || tile.px[1] < 0 {
                    continue;
                }

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
