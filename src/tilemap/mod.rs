use bevy::app::{Plugin, Update};

use self::{
    chunking::camera::{CameraChunkUpdater, CameraChunkUpdation},
    map::{
        TilePivot, TileRenderSize, TilemapAnimations, TilemapLayerOpacities, TilemapName,
        TilemapSlotSize, TilemapStorage, TilemapTexture, TilemapTextureDescriptor,
        TilemapTransform, TilemapType,
    },
    tile::{LayerUpdater, Tile, TileLayer, TileTexture, TileUpdater},
};

#[cfg(feature = "algorithm")]
pub mod algorithm;
pub mod buffers;
pub mod bundles;
pub mod chunking;
pub mod coordinates;
pub mod map;
#[cfg(any(feature = "physics_xpbd", feature = "physics_rapier"))]
pub mod physics;
pub mod tile;

pub struct EntiTilesTilemapPlugin;

impl Plugin for EntiTilesTilemapPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(
            Update,
            (
                map::transform_syncer,
                map::tilemap_aabb_calculator,
                tile::tile_updater,
                chunking::camera::camera_chunk_update,
            ),
        );

        app.register_type::<TileLayer>()
            .register_type::<LayerUpdater>()
            .register_type::<TileUpdater>()
            .register_type::<Tile>()
            .register_type::<TileTexture>();

        app.register_type::<TilemapName>()
            .register_type::<TileRenderSize>()
            .register_type::<TilemapSlotSize>()
            .register_type::<TilemapType>()
            .register_type::<TilePivot>()
            .register_type::<TilemapLayerOpacities>()
            .register_type::<TilemapStorage>()
            .register_type::<TilemapTransform>()
            .register_type::<TilemapTexture>()
            .register_type::<TilemapTextureDescriptor>()
            .register_type::<TilemapAnimations>();

        app.register_type::<CameraChunkUpdation>()
            .register_type::<CameraChunkUpdater>();

        app.add_event::<CameraChunkUpdation>();

        #[cfg(feature = "algorithm")]
        app.add_plugins(algorithm::EntiTilesAlgorithmTilemapPlugin);
        #[cfg(any(feature = "physics_rapier", feature = "physics_xpbd"))]
        app.add_plugins(physics::EntiTilesPhysicsTilemapPlugin);
    }
}
