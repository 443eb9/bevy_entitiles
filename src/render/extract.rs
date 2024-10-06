use bevy::{
    asset::{AssetId, Handle},
    ecs::{
        entity::EntityHashMap,
        event::EventReader,
        query::QueryItem,
        system::{lifetimeless::Read, Res},
    },
    prelude::{Changed, Commands, Component, DetectChanges, Entity, Query, Ref, Vec2, Vec4},
    render::{
        extract_instances::{ExtractInstance, ExtractedInstances},
        Extract,
    },
};

use crate::{
    math::CameraAabb2d,
    render::{
        chunk::{ChunkUnload, RenderChunkSort, UnloadRenderChunk},
        cull::FrustumCulling,
    },
    tilemap::{
        despawn::{DespawnedTile, DespawnedTilemap},
        map::{
            TilePivot, TileRenderSize, TilemapAnimations, TilemapAxisFlip, TilemapLayerOpacities,
            TilemapName, TilemapSlotSize, TilemapStorage, TilemapTextures, TilemapTransform,
            TilemapType,
        },
        tile::Tile,
    },
};

pub type TilemapInstances = ExtractedInstances<ExtractedTilemap>;

pub type TilemapMaterialInstances<M> = ExtractedInstances<AssetId<M>>;

#[derive(Component, Debug)]
pub struct ExtractedTilemap {
    pub name: String,
    pub tile_render_size: Vec2,
    pub slot_size: Vec2,
    pub ty: TilemapType,
    pub tile_pivot: Vec2,
    pub layer_opacities: Vec4,
    pub transform: TilemapTransform,
    pub axis_flip: TilemapAxisFlip,
    pub texture: Option<Handle<TilemapTextures>>,
    pub changed_animations: Option<TilemapAnimations>,
    pub chunk_size: u32,
}

impl ExtractInstance for ExtractedTilemap {
    type QueryData = (
        Read<TilemapName>,
        Read<TileRenderSize>,
        Read<TilemapSlotSize>,
        Read<TilemapType>,
        Read<TilePivot>,
        Read<TilemapLayerOpacities>,
        Read<TilemapTransform>,
        Read<TilemapAxisFlip>,
        Read<TilemapStorage>,
        Option<Read<Handle<TilemapTextures>>>,
        Option<Ref<'static, TilemapAnimations>>,
    );

    type QueryFilter = ();

    fn extract(item: QueryItem<'_, Self::QueryData>) -> Option<Self> {
        let (
            name,
            tile_render_size,
            slot_size,
            ty,
            tile_pivot,
            layer_opacities,
            transform,
            axis_flip,
            storage,
            texture,
            animations,
        ) = item;
        assert_ne!(
            storage.tilemap,
            Entity::PLACEHOLDER,
            "You are trying to spawn a tilemap that has a invalid storage! \
            Did you use the default storage? If so, you have to assign the valid \
            entity for the storage when creating."
        );

        Some(ExtractedTilemap {
            name: name.0.clone(),
            tile_render_size: tile_render_size.0,
            slot_size: slot_size.0,
            ty: *ty,
            tile_pivot: tile_pivot.0,
            layer_opacities: layer_opacities.0,
            transform: *transform,
            axis_flip: *axis_flip,
            texture: texture.cloned(),
            changed_animations: animations
                .as_ref()
                .is_some_and(|a| a.is_changed())
                .then(|| animations.unwrap().clone()),
            chunk_size: storage.storage.chunk_size,
        })
    }
}

pub type ExtractedTile = Tile;

pub type ExtractedView = CameraAabb2d;

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

pub fn extract_resources(
    mut commands: Commands,
    frustum_culling: Extract<Res<FrustumCulling>>,
    sort_config: Extract<Res<RenderChunkSort>>,
) {
    commands.insert_resource(FrustumCulling(frustum_culling.0));
    commands.insert_resource(sort_config.clone());
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
