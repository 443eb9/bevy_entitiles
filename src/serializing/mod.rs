use std::{fs::File, io::Write, marker::PhantomData, path::Path};

use bevy::app::Plugin;
use ron::error::SpannedError;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::render::material::TilemapMaterial;

pub mod chunk;
pub mod map;
pub mod pattern;

#[derive(Default)]
pub struct EntiTilesSerializingPlugin<M: TilemapMaterial + Serialize + DeserializeOwned>(
    PhantomData<M>,
);

impl<M: TilemapMaterial + Serialize + DeserializeOwned> Plugin for EntiTilesSerializingPlugin<M> {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins((
            chunk::EntiTilesChunkSerializingPlugin,
            map::EntiTilesTilemapSerializingPlugin::<M>::default(),
        ));
    }
}

pub fn save_object<T: Serialize>(path: &Path, file_name: &str, object: &T) {
    std::fs::create_dir_all(path).unwrap_or_else(|err| panic!("{:?}", err));
    let path = path.join(file_name);
    File::create(path.clone())
        .unwrap_or(File::open(path).unwrap())
        .write(ron::to_string(object).unwrap().as_bytes())
        .unwrap_or_else(|err| panic!("{:?}", err));
}

pub fn load_object<T: for<'a> Deserialize<'a>>(
    path: &Path,
    file_name: &str,
) -> Result<T, SpannedError> {
    ron::from_str(std::fs::read_to_string(path.join(file_name))?.as_str())
}
