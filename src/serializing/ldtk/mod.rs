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
    render::texture::{TilemapTexture, TilemapTextureDescriptor},
    tilemap::{
        map::{Tilemap, TilemapBuilder},
        tile::TileType,
    },
};

use self::{definitions::TilesetDef, json::LdtkJson};

pub mod definitions;
pub mod json;
pub mod level;

#[derive(Component)]
pub struct LdtkLoader {
    pub path: String,
    pub tilemap_name: String,
    pub scale: f32,
    pub filter_mode: FilterMode,
}

pub fn load_ldtk(
    commands: ParallelCommands,
    mut loader_query: Query<(Entity, &LdtkLoader)>,
    asset_server: Res<AssetServer>,
) {
    loader_query.par_iter_mut().for_each(|(entity, loader)| {
        let Ok(str_raw) = read_to_string(&loader.path) else {
            panic!("Could not read file at path: {}", loader.path);
        };

        let Ok(ldtk_data) = serde_json::from_str::<LdtkJson>(&str_raw) else {
            panic!("Could not parse file at path: {}", loader.path);
        };

        // texture
        assert_eq!(
            ldtk_data.defs.tilesets.len(),
            1,
            "Multiple tilesets are not supported yet"
        );
        let tileset = &ldtk_data.defs.tilesets[0];
        let texture = load_texture(tileset, loader.filter_mode, &asset_server);

        // level
        let mut max_size = UVec2::ZERO;
        for level in ldtk_data.levels.iter() {
            for layer in level.layer_instances.iter() {
                max_size.x = max_size.x.max(layer.c_wid as u32);
                max_size.y = max_size.y.max(layer.c_hei as u32);
            }
        }

        // let (tilemap_entity, tilemap) = TilemapBuilder::new(
        //     TileType::Square,
        //     max_size,
        //     tileset.tile_grid_size as f32 * loader.scale * Vec2::ONE,
        //     loader.tilemap_name,
        // ).with_texture(texture).;

        commands.command_scope(|mut c| {
            c.entity(entity).remove::<LdtkLoader>();
        });
    });
}

fn load_texture(
    tileset: &TilesetDef,
    filter_mode: FilterMode,
    asset_server: &Res<AssetServer>,
) -> TilemapTexture {
    let texture = asset_server.load(tileset.rel_path.clone().unwrap());
    let desc = TilemapTextureDescriptor::from_full_grid(
        UVec2 {
            x: tileset.px_wid as u32,
            y: tileset.px_hei as u32,
        },
        UVec2 {
            x: tileset.c_wid as u32,
            y: tileset.c_wid as u32,
        },
        filter_mode,
    );
    TilemapTexture { texture, desc }
}
