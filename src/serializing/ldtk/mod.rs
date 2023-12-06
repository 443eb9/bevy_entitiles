use std::{
    fs::{read_to_string, File},
    io::Write,
};

use bevy::ecs::{
    component::Component,
    entity::Entity,
    system::{Commands, ParallelCommands, Query},
};

use self::json::LdtkJson;

pub mod definitions;
pub mod instances;
pub mod json;
pub mod level;

#[derive(Component)]
pub struct LdtkLoader {
    pub path: String,
}

pub fn load_ldtk(mut commands: ParallelCommands, mut loader_query: Query<(Entity, &LdtkLoader)>) {
    loader_query.par_iter_mut().for_each(|(entity, loader)| {
        let Ok(str_raw) = read_to_string(&loader.path) else {
            panic!("Could not read file at path: {}", loader.path);
        };

        let Ok(ldtk_raw) = serde_json::from_str::<LdtkJson>(&str_raw) else {
            panic!("Could not parse file at path: {}", loader.path);
        };

        commands.command_scope(|mut c| {
            c.entity(entity).remove::<LdtkLoader>();
        })
    });
}
