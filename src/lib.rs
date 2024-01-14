use bevy::{app::Update, prelude::Plugin};
use math::EntiTilesMathPlugin;
use render::{texture, EntiTilesRendererPlugin};
use tilemap::EntiTilesTilemapPlugin;

#[cfg(feature = "algorithm")]
pub mod algorithm;
#[cfg(feature = "debug")]
pub mod debug;
#[cfg(feature = "ldtk")]
pub mod ldtk;
pub mod math;
pub mod render;
#[cfg(feature = "serializing")]
pub mod serializing;
pub mod tilemap;

pub const MAX_TILESET_COUNT: usize = 4;
pub const MAX_LAYER_COUNT: usize = 4;
pub const MAX_ATLAS_COUNT: usize = 512;
pub const MAX_ANIM_COUNT: usize = 64;
pub const MAX_ANIM_SEQ_LENGTH: usize = 16;
pub const DEFAULT_CHUNK_SIZE: u32 = 32;

pub mod prelude {
    #[cfg(feature = "algorithm")]
    pub use crate::algorithm::{
        pathfinding::{AsyncPathfinder, Path, Pathfinder},
        wfc::WfcRunner,
    };
    #[cfg(feature = "ldtk")]
    pub use crate::ldtk::resources::{LdtkAssets, LdtkLevelManager};
    pub use crate::math::{aabb::Aabb2d, TileArea};
    #[cfg(feature = "serializing")]
    pub use crate::serializing::{map::load::TilemapLoaderBuilder, map::save::TilemapSaver};
    #[cfg(feature = "physics")]
    pub use crate::tilemap::physics::TileCollision;
    pub use crate::tilemap::{
        bundles::{PureColorTilemapBundle, TilemapBundle},
        map::{
            TilePivot, TileRenderSize, TilemapAnimations, TilemapLayerOpacities, TilemapName,
            TilemapSlotSize, TilemapStorage, TilemapTexture, TilemapTextureDescriptor,
            TilemapTransform, TilemapType,
        },
        tile::{TileAnimation, TileBuilder, TileLayer, TileUpdater},
    };
}

pub struct EntiTilesPlugin;

impl Plugin for EntiTilesPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Update, texture::set_texture_usage);

        app.add_plugins((
            EntiTilesTilemapPlugin,
            EntiTilesRendererPlugin,
            EntiTilesMathPlugin,
            #[cfg(feature = "debug")]
            debug::EntiTilesDebugPlugin,
            #[cfg(feature = "algorithm")]
            algorithm::EntiTilesAlgorithmPlugin,
            #[cfg(feature = "serializing")]
            serializing::EntiTilesSerializingPlugin,
            #[cfg(feature = "ldtk")]
            ldtk::EntiTilesLdtkPlugin,
        ));
    }
}
