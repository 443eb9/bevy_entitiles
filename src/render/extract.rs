use bevy::{
    asset::{AssetEvent, AssetId, Assets, Handle},
    ecs::{
        entity::EntityHashMap,
        event::EventReader,
        query::{Or, With},
        system::{Res, ResMut},
    },
    prelude::{Changed, Commands, Component, Entity, Query, Vec2, Vec4},
    render::{view::InheritedVisibility, Extract},
    utils::HashSet,
};

use crate::{
    math::CameraAabb2d,
    tilemap::{
        despawn::{DespawnedTile, DespawnedTilemap},
        map::{
            TilePivot, TileRenderSize, TilemapAnimations, TilemapAxisFlip, TilemapLayerOpacities,
            TilemapName, TilemapSlotSize, TilemapStorage, TilemapTransform, TilemapType,
        },
        tile::Tile,
    },
};

use super::{
    chunk::{ChunkUnload, UnloadRenderChunk},
    cull::FrustumCulling,
    material::{ExtractedStandardTilemapMaterials, StandardTilemapMaterial},
    resources::TilemapInstances,
};

#[derive(Component, Debug)]
pub struct TilemapInstance;

#[derive(Component, Debug)]
pub struct ExtractedTilemap {
    pub id: Entity,
    pub name: String,
    pub tile_render_size: Vec2,
    pub slot_size: Vec2,
    pub ty: TilemapType,
    pub tile_pivot: Vec2,
    pub layer_opacities: Vec4,
    pub transform: TilemapTransform,
    pub axis_flip: TilemapAxisFlip,
    pub material: AssetId<StandardTilemapMaterial>,
    pub animations: Option<TilemapAnimations>,
    pub chunk_size: u32,
}

pub type ExtractedTile = Tile;

pub type ExtractedView = CameraAabb2d;

pub fn extract_changed_tilemaps(
    tilemaps_query: Extract<
        Query<
            (
                Entity,
                &TilemapName,
                &TileRenderSize,
                &TilemapSlotSize,
                &TilemapType,
                &TilePivot,
                &TilemapLayerOpacities,
                &TilemapTransform,
                &TilemapAxisFlip,
                &TilemapStorage,
                &Handle<StandardTilemapMaterial>,
                Option<&TilemapAnimations>,
            ),
            Or<(
                Changed<TileRenderSize>,
                Changed<TilemapSlotSize>,
                Changed<TilemapType>,
                Changed<TilePivot>,
                Changed<TilemapLayerOpacities>,
                Changed<TilemapTransform>,
                Changed<TilemapAxisFlip>,
                Changed<Handle<StandardTilemapMaterial>>,
                Changed<TilemapAnimations>,
            )>,
        >,
    >,
    mut instances: ResMut<TilemapInstances>,
) {
    tilemaps_query.iter().for_each(
        |(
            entity,
            name,
            tile_render_size,
            slot_size,
            ty,
            tile_pivot,
            layer_opacities,
            transform,
            axis_flip,
            storage,
            material,
            animations,
        )| {
            assert_ne!(
                storage.tilemap,
                Entity::PLACEHOLDER,
                "You are trying to spawn a tilemap that has a invalid storage! \
                Did you use the default storage? If so, you have to assign the valid \
                entity for the storage when creating."
            );
            instances.0.insert(
                entity,
                ExtractedTilemap {
                    id: entity,
                    name: name.0.clone(),
                    tile_render_size: tile_render_size.0,
                    slot_size: slot_size.0,
                    ty: *ty,
                    tile_pivot: tile_pivot.0,
                    layer_opacities: layer_opacities.0,
                    transform: *transform,
                    axis_flip: *axis_flip,
                    material: material.id(),
                    animations: animations.cloned(),
                    chunk_size: storage.storage.chunk_size,
                },
            );
        },
    );
}

// From bevy_sprite::mesh2d::material::extract_materials_2d
pub fn extract_std_materials(
    mut commands: Commands,
    mut events: Extract<EventReader<AssetEvent<StandardTilemapMaterial>>>,
    assets: Extract<Res<Assets<StandardTilemapMaterial>>>,
) {
    let mut changed_assets = HashSet::default();
    let mut removed = Vec::new();
    for event in events.read() {
        #[allow(clippy::match_same_arms)]
        match event {
            AssetEvent::Added { id } | AssetEvent::Modified { id } => {
                changed_assets.insert(*id);
            }
            AssetEvent::Removed { id } => {
                changed_assets.remove(id);
                removed.push(*id);
            }
            AssetEvent::Unused { .. } => {}
            AssetEvent::LoadedWithDependencies { .. } => {
                // TODO: handle this
            }
        }
    }

    let mut extracted_assets = Vec::new();
    for id in changed_assets.drain() {
        if let Some(asset) = assets.get(id) {
            extracted_assets.push((id, asset.clone()));
        }
    }

    commands.insert_resource(ExtractedStandardTilemapMaterials {
        extracted: extracted_assets,
        removed,
    });
}

pub fn extract_tilemaps(
    mut commands: Commands,
    tilemaps_query: Extract<Query<(Entity, &InheritedVisibility), With<TilemapStorage>>>,
) {
    commands.insert_or_spawn_batch(
        tilemaps_query
            .iter()
            .filter_map(|(entity, inherited_visibility)| {
                if inherited_visibility.get() {
                    Some((entity, TilemapInstance))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>(),
    );
}

pub fn extract_tiles(
    mut commands: Commands,
    tiles_query: Extract<Query<(Entity, &Tile), Changed<Tile>>>,
) {
    commands.insert_or_spawn_batch(
        tiles_query
            .iter()
            .map(|(entity, tile)| {
                (
                    entity,
                    ExtractedTile {
                        tilemap_id: tile.tilemap_id,
                        chunk_index: tile.chunk_index,
                        in_chunk_index: tile.in_chunk_index,
                        index: tile.index,
                        texture: tile.texture.clone(),
                        tint: tile.tint,
                    },
                )
            })
            .collect::<Vec<_>>(),
    );
}

pub fn extract_view(
    mut commands: Commands,
    cameras: Extract<Query<(Entity, &CameraAabb2d), Changed<CameraAabb2d>>>,
) {
    commands.insert_or_spawn_batch(
        cameras
            .iter()
            .map(|(e, aabb)| (e, *aabb))
            .collect::<Vec<_>>(),
    );
}

pub fn extract_unloaded_chunks(
    mut commands: Commands,
    mut chunk_unload: Extract<EventReader<ChunkUnload>>,
) {
    commands.insert_or_spawn_batch(chunk_unload.read().fold(
        EntityHashMap::<UnloadRenderChunk>::default(),
        |mut acc, elem| {
            acc.entry(elem.tilemap).or_default().0.push(elem.index);
            acc
        },
    ));
}

pub fn extract_resources(mut commands: Commands, frustum_culling: Extract<Res<FrustumCulling>>) {
    commands.insert_resource(FrustumCulling(frustum_culling.0));
}

pub fn extract_despawned_tilemaps(
    mut commands: Commands,
    tilemaps_query: Extract<Query<(Entity, &DespawnedTilemap)>>,
) {
    let mut despawned_tilemaps = Vec::new();

    tilemaps_query.iter().for_each(|(entity, map)| {
        despawned_tilemaps.push((entity, map.clone()));
    });

    commands.insert_or_spawn_batch(despawned_tilemaps);
}

pub fn extract_despawned_tiles(
    mut commands: Commands,
    tiles_query: Extract<Query<(Entity, &DespawnedTile)>>,
) {
    let mut despawned_tiles = Vec::new();

    tiles_query.iter().for_each(|(entity, tile)| {
        despawned_tiles.push((entity, tile.clone()));
    });

    commands.insert_or_spawn_batch(despawned_tiles);
}
