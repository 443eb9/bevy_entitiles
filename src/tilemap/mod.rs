use bevy::{
    app::{Plugin, PostUpdate, PreUpdate, Update},
    asset::AssetApp,
};

use crate::tilemap::{
    chunking::camera::{CameraChunkUpdater, CameraChunkUpdation},
    map::{
        TilePivot, TileRenderSize, TilemapAabbs, TilemapAnimations, TilemapLayerOpacities,
        TilemapName, TilemapSlotSize, TilemapStorage, TilemapTexture, TilemapTextureDescriptor,
        TilemapTextures, TilemapTransform, TilemapType,
    },
    tile::{LayerUpdater, Tile, TileLayer, TileTexture, TileUpdater},
};

#[cfg(feature = "algorithm")]
pub mod algorithm;
pub mod buffers;
pub mod bundles;
pub mod chunking;
pub mod coordinates;
pub mod despawn;
pub mod map;
#[cfg(feature = "physics")]
pub mod physics;
pub mod tile;

pub struct EntiTilesTilemapPlugin;

impl Plugin for EntiTilesTilemapPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(PreUpdate, despawn::despawn_component_remover)
            .add_systems(
                Update,
                (
                    map::transform_syncer,
                    map::queued_chunk_aabb_calculator,
                    map::tilemap_aabb_calculator,
                    tile::tile_updater,
                    tile::tile_rearranger,
                    chunking::camera::camera_chunk_update,
                ),
            )
            .add_systems(
                PostUpdate,
                (
                    despawn::despawn_tilemap,
                    despawn::despawn_tiles,
                    #[cfg(feature = "physics")]
                    despawn::despawn_physics_tilemaps,
                ),
            )
            .register_type::<TileLayer>()
            .register_type::<LayerUpdater>()
            .register_type::<TileUpdater>()
            .register_type::<Tile>()
            .register_type::<TileTexture>()
            .register_type::<TilemapName>()
            .register_type::<TileRenderSize>()
            .register_type::<TilemapSlotSize>()
            .register_type::<TilemapType>()
            .register_type::<TilePivot>()
            .register_type::<TilemapLayerOpacities>()
            .register_type::<TilemapStorage>()
            .register_type::<TilemapAabbs>()
            .register_type::<TilemapTransform>()
            .register_type::<TilemapTexture>()
            .register_type::<TilemapTextureDescriptor>()
            .register_type::<TilemapAnimations>()
            .register_type::<CameraChunkUpdation>()
            .register_type::<CameraChunkUpdater>()
            .init_asset::<TilemapTextures>()
            .add_event::<CameraChunkUpdation>();

        #[cfg(feature = "algorithm")]
        app.add_plugins(algorithm::EntiTilesAlgorithmTilemapPlugin);
        #[cfg(feature = "physics")]
        app.add_plugins(physics::EntiTilesPhysicsTilemapPlugin);
    }
}
