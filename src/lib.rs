use bevy::{app::Update, prelude::Plugin};
use math::{aabb::AabbBox2d, FillArea};
use reflect::ReflectFilterMode;
use render::{texture, EntiTilesRendererPlugin};
use tilemap::{
    layer::{LayerInserter, LayerUpdater, TileLayer},
    map::{Tilemap, TilemapTransform},
    tile::{AnimatedTile, Tile},
    EntiTilesTilemapPlugin,
};

#[cfg(feature = "algorithm")]
pub mod algorithm;
#[cfg(feature = "debug")]
pub mod debug;
#[cfg(feature = "ldtk")]
pub mod ldtk;
pub mod math;
pub mod reflect;
pub mod render;
#[cfg(feature = "serializing")]
pub mod serializing;
pub mod tilemap;
pub mod ui;

pub const MAX_TILESET_COUNT: usize = 4;
pub const MAX_LAYER_COUNT: usize = 4;
pub const MAX_ATLAS_COUNT: usize = 512;
pub const MAX_ANIM_COUNT: usize = 64;
pub const MAX_ANIM_SEQ_LENGTH: usize = 16;

pub struct EntiTilesPlugin;

impl Plugin for EntiTilesPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Update, texture::set_texture_usage);

        app.add_plugins((EntiTilesTilemapPlugin, EntiTilesRendererPlugin));

        #[cfg(feature = "algorithm")]
        app.add_plugins(algorithm::EntiTilesAlgorithmPlugin);
        #[cfg(feature = "serializing")]
        app.add_plugins(serializing::EntiTilesSerializingPlugin);
        #[cfg(feature = "ldtk")]
        app.add_plugins(ldtk::EntiTilesLdtkPlugin);
        #[cfg(feature = "ui")]
        app.add_plugins(ui::EntiTilesUiPlugin);

        app.register_type::<AabbBox2d>().register_type::<FillArea>();

        app.register_type::<TileLayer>()
            .register_type::<LayerInserter>()
            .register_type::<LayerUpdater>()
            .register_type::<TilemapTransform>()
            .register_type::<Tilemap>()
            .register_type::<Tile>()
            .register_type::<AnimatedTile>();

        app.register_type::<ReflectFilterMode>();
    }
}
