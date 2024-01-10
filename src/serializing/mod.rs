use std::{fs::File, io::Write, path::Path};

use bevy::app::{Plugin, Update};
use ron::error::SpannedError;
use serde::{Deserialize, Serialize};

use self::{
    chunk::{
        camera::{CameraChunkUpdater, CameraChunkUpdation},
        save::TilemapColorChunkSaver,
    },
    map::{
        load::{TilemapLoadFailure, TilemapLoader},
        save::TilemapSaver,
        SerializedTilemap, SerializedTilemapData, SerializedTilemapTexture,
        SerializedTilemapTextureDescriptor,
    },
};

pub mod chunk;
pub mod map;
pub mod pattern;

pub struct EntiTilesSerializingPlugin;

impl Plugin for EntiTilesSerializingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(
            Update,
            (
                map::save::save,
                map::load::load,
                chunk::save::save_color_layer,
                #[cfg(feature = "algorithm")]
                chunk::save::save_path_layer,
                chunk::save::render_chunk_remover,
                chunk::save::saver_expander,
                chunk::load::loader_expander,
                chunk::load::load_color_layer,
                #[cfg(feature = "algorithm")]
                chunk::load::load_path_layer,
                chunk::camera::camera_chunk_update,
            ),
        );

        app.register_type::<TilemapLoader>()
            .register_type::<TilemapSaver>()
            .register_type::<TilemapLoadFailure>()
            .register_type::<SerializedTilemapData>()
            .register_type::<SerializedTilemap>()
            .register_type::<SerializedTilemapTextureDescriptor>()
            .register_type::<SerializedTilemapTexture>();

        app.register_type::<TilemapColorChunkSaver>()
            .register_type::<CameraChunkUpdater>();

        app.add_event::<CameraChunkUpdation>();

        #[cfg(feature = "algorithm")]
        {
            app.register_type::<chunk::save::TilemapPathChunkSaver>();

            app.add_systems(Update, chunk::save::save_path_layer);
        }
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
