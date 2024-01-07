use bevy::app::{Plugin, Update};

use self::map::{
    load::{TilemapLoadFailure, TilemapLoader},
    save::TilemapSaver,
    SerializedTilemap, SerializedTilemapData, SerializedTilemapTextureDescriptor,
    SerializedTilemapTexture,
};

pub mod chunk;
pub mod map;

pub struct EntiTilesSerializingPlugin;

impl Plugin for EntiTilesSerializingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Update, (map::save::save, map::load::load));

        app.register_type::<TilemapLoader>()
            .register_type::<TilemapSaver>()
            .register_type::<TilemapLoadFailure>()
            .register_type::<SerializedTilemapData>()
            .register_type::<SerializedTilemap>()
            .register_type::<SerializedTilemapTextureDescriptor>()
            .register_type::<SerializedTilemapTexture>();
    }
}
