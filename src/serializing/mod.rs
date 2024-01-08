use bevy::app::{Plugin, Update};

use self::{
    chunk::{save::TilemapChunkUnloader, TilemapChunkCache},
    map::{
        load::{TilemapLoadFailure, TilemapLoader},
        save::TilemapSaver,
        SerializedTilemap, SerializedTilemapData, SerializedTilemapTexture,
        SerializedTilemapTextureDescriptor,
    },
};

pub mod chunk;
pub mod map;

pub struct EntiTilesSerializingPlugin;

impl Plugin for EntiTilesSerializingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(
            Update,
            (map::save::save, map::load::load, chunk::save::save),
        );

        app.register_type::<TilemapLoader>()
            .register_type::<TilemapSaver>()
            .register_type::<TilemapLoadFailure>()
            .register_type::<SerializedTilemapData>()
            .register_type::<SerializedTilemap>()
            .register_type::<SerializedTilemapTextureDescriptor>()
            .register_type::<SerializedTilemapTexture>();

        app.register_type::<TilemapChunkUnloader>()
            .register_type::<TilemapChunkCache>();

        app.init_resource::<TilemapChunkCache>();

        #[cfg(feature = "algorithm")]
        {
            app.register_type::<chunk::TilemapPathChunkCache>();
            
            app.init_resource::<chunk::TilemapPathChunkCache>();
        }
    }
}
