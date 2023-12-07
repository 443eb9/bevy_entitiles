use std::{
    fs::{read_to_string, File},
    io::Write,
};

use bevy::{
    asset::AssetServer,
    ecs::{
        component::Component,
        entity::Entity,
        system::{Commands, ParallelCommands, Query, Res},
    },
    math::UVec2,
    render::render_resource::FilterMode,
};

use crate::render::texture::{TilemapTexture, TilemapTextureDescriptor};

use self::{definitions::TilesetDef, json::LdtkJson};

pub mod definitions;
pub mod instances;
pub mod json;
pub mod level;

#[derive(Component)]
pub struct LdtkLoader {
    pub path: String,
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
        let texture = load_texture(
            &ldtk_data.defs.tilesets[0],
            loader.filter_mode,
            &asset_server,
        );

        // level
        for level in ldtk_data.levels.iter() {
            
        }

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
