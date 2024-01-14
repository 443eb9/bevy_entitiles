use std::{f32::consts::SQRT_2, fmt::Debug};

use bevy::{
    asset::Handle,
    ecs::{
        component::Component,
        query::Changed,
        system::{ParallelCommands, Query},
    },
    math::{Mat2, Quat, Vec4},
    prelude::{Assets, Commands, Entity, IVec2, Image, ResMut, UVec2, Vec2},
    reflect::Reflect,
    render::render_resource::{FilterMode, TextureUsages},
    sprite::TextureAtlas,
    transform::components::Transform,
};

use crate::math::{
    aabb::{Aabb2d, IAabb2d},
    TileArea,
};

use super::{
    buffers::TileBuilderBuffer,
    chunking::storage::ChunkedStorage,
    despawn::DespawnMe,
    tile::{TileAnimation, TileBuilder, TileUpdater},
};

/// Defines the shape of tiles in a tilemap.
/// Check the `Coordinate Systems` chapter in README.md to see the details.
#[derive(Default, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect, Component)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub enum TilemapType {
    #[default]
    Square,
    Isometric,
    Hexagonal(u32),
}

#[derive(Debug, Clone, Copy, Default, Reflect)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub enum TilemapRotation {
    #[default]
    None = 0,
    Cw90 = 90,
    Cw180 = 180,
    Cw270 = 270,
}

#[derive(Component, Debug, Clone, Copy, Default, Reflect)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub struct TilemapTransform {
    pub translation: Vec2,
    pub z_index: i32,
    pub rotation: TilemapRotation,
}

impl TilemapTransform {
    pub const IDENTITY: Self = Self {
        translation: Vec2::ZERO,
        z_index: 0,
        rotation: TilemapRotation::None,
    };

    #[inline]
    pub fn from_translation(translation: Vec2) -> Self {
        Self {
            translation,
            ..Default::default()
        }
    }

    #[inline]
    pub fn from_translation_3d(translation: Vec2, z: i32) -> Self {
        Self {
            translation,
            z_index: z,
            ..Default::default()
        }
    }

    #[inline]
    pub fn from_z_index(z: i32) -> Self {
        Self {
            z_index: z,
            ..Default::default()
        }
    }

    #[inline]
    pub fn transform_point(&self, point: Vec2) -> Vec2 {
        self.apply_translation(self.apply_rotation(point))
    }

    pub fn transform_aabb(&self, aabb: Aabb2d) -> Aabb2d {
        let min = self.transform_point(aabb.min);
        let max = self.transform_point(aabb.max);

        match self.rotation {
            TilemapRotation::None => Aabb2d { min, max },
            TilemapRotation::Cw90 => Aabb2d::new(max.x, min.y, min.x, max.y),
            TilemapRotation::Cw180 => Aabb2d::new(max.x, max.y, min.x, min.y),
            TilemapRotation::Cw270 => Aabb2d::new(min.x, max.y, max.x, min.y),
        }
    }

    #[inline]
    pub fn get_rotation_matrix(&self) -> Mat2 {
        match self.rotation {
            TilemapRotation::None => Mat2::from_cols_array(&[1., 0., 0., 1.]),
            TilemapRotation::Cw90 => Mat2::from_cols_array(&[0., 1., -1., 0.]),
            TilemapRotation::Cw180 => Mat2::from_cols_array(&[-1., 0., 0., -1.]),
            TilemapRotation::Cw270 => Mat2::from_cols_array(&[0., -1., 1., 0.]),
        }
    }

    #[inline]
    pub fn get_rotation_quat(&self) -> Quat {
        match self.rotation {
            TilemapRotation::None => Quat::IDENTITY,
            TilemapRotation::Cw90 => Quat::from_xyzw(0., 0., SQRT_2 / 2., SQRT_2 / 2.),
            TilemapRotation::Cw180 => Quat::from_xyzw(0., 0., 1., 0.),
            TilemapRotation::Cw270 => Quat::from_xyzw(0., 0., SQRT_2 / 2., -SQRT_2 / 2.),
        }
    }

    #[inline]
    pub fn apply_rotation(&self, point: Vec2) -> Vec2 {
        match self.rotation {
            TilemapRotation::None => point,
            TilemapRotation::Cw90 => Vec2::new(-point.y, point.x),
            TilemapRotation::Cw180 => Vec2::new(-point.x, -point.y),
            TilemapRotation::Cw270 => Vec2::new(point.y, -point.x),
        }
    }

    #[inline]
    pub fn apply_translation(&self, point: Vec2) -> Vec2 {
        point + self.translation
    }
}

impl Into<Transform> for TilemapTransform {
    fn into(self) -> Transform {
        Transform {
            translation: self.translation.extend(self.z_index as f32),
            rotation: Quat::from_rotation_z(self.rotation as u32 as f32),
            ..Default::default()
        }
    }
}

#[derive(Component, Clone, Default, Debug, Reflect)]
pub struct TilemapTexture {
    pub(crate) texture: Handle<Image>,
    pub(crate) desc: TilemapTextureDescriptor,
    pub(crate) rotation: TilemapRotation,
}

impl TilemapTexture {
    pub fn new(
        texture: Handle<Image>,
        desc: TilemapTextureDescriptor,
        rotation: TilemapRotation,
    ) -> Self {
        Self {
            texture,
            desc,
            rotation,
        }
    }

    pub fn clone_weak(&self) -> Handle<Image> {
        self.texture.clone_weak()
    }

    pub fn desc(&self) -> &TilemapTextureDescriptor {
        &self.desc
    }

    pub fn handle(&self) -> &Handle<Image> {
        &self.texture
    }

    pub fn as_texture_atlas(&self) -> TextureAtlas {
        TextureAtlas::from_grid(
            self.texture.clone(),
            self.desc.tile_size.as_vec2(),
            self.desc.size.x as usize,
            self.desc.size.y as usize,
            Some(Vec2::ZERO),
            Some(Vec2::ZERO),
        )
    }

    /// Bevy doesn't set the `COPY_SRC` usage for images by default, so we need to do it manually.
    pub(crate) fn set_usage(&mut self, image_assets: &mut ResMut<Assets<Image>>) {
        let Some(image) = image_assets.get(&self.clone_weak()) else {
            return;
        };

        if !image
            .texture_descriptor
            .usage
            .contains(TextureUsages::COPY_SRC)
        {
            image_assets
                .get_mut(&self.clone_weak())
                .unwrap()
                .texture_descriptor
                .usage
                .set(TextureUsages::COPY_SRC, true);
        }
    }
}

#[derive(Clone, Default, Debug, PartialEq, Reflect)]
pub struct TilemapTextureDescriptor {
    pub(crate) size: UVec2,
    pub(crate) tile_size: UVec2,
    #[reflect(ignore)]
    pub(crate) filter_mode: FilterMode,
}

impl TilemapTextureDescriptor {
    pub fn new(size: UVec2, tile_size: UVec2, filter_mode: FilterMode) -> Self {
        assert_eq!(
            size % tile_size,
            UVec2::ZERO,
            "Invalid tilemap texture descriptor! The size must be divisible by the tile size! \
            If the spare pixels are not needed and you are not using this texture for ui, \
            you can \"lie\" to the descriptor by setting the size to fit the tiles."
        );

        Self {
            size,
            tile_size,
            filter_mode,
        }
    }
}

#[derive(Component, Default, Debug, Clone, Reflect)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub struct TilemapName(pub String);

#[derive(Component, Default, Debug, Clone, Copy, Reflect)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub struct TileRenderSize(pub Vec2);

#[derive(Component, Default, Debug, Clone, Copy, Reflect)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub struct TilemapSlotSize(pub Vec2);

#[derive(Component, Default, Debug, Clone, Copy, Reflect)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub struct TilePivot(pub Vec2);

#[derive(Component, Debug, Clone, Copy, Reflect)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub struct TilemapLayerOpacities(pub Vec4);

impl Default for TilemapLayerOpacities {
    fn default() -> Self {
        Self(Vec4::ONE)
    }
}

#[derive(Component, Default, Debug, Clone, Copy, Reflect)]
pub struct TilemapAabbs {
    pub(crate) chunk_aabb: IAabb2d,
    pub(crate) world_aabb: Aabb2d,
}

#[derive(Component, Debug, Clone, Reflect)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub struct TilemapStorage {
    pub(crate) tilemap: Entity,
    pub(crate) storage: ChunkedStorage<Entity>,
}

impl TilemapStorage {
    pub fn new(chunk_size: u32, binded_tilemap: Entity) -> Self {
        Self {
            tilemap: binded_tilemap,
            storage: ChunkedStorage::new(chunk_size),
            ..Default::default()
        }
    }
}

impl Default for TilemapStorage {
    fn default() -> Self {
        Self {
            tilemap: Entity::PLACEHOLDER,
            storage: Default::default(),
        }
    }
}

impl TilemapStorage {
    /// Get a tile.
    #[inline]
    pub fn get(&self, index: IVec2) -> Option<Entity> {
        self.storage.get_elem(index).cloned()
    }

    /// Get a chunk.
    #[inline]
    pub fn get_chunk(&self, index: IVec2) -> Option<&Vec<Option<Entity>>> {
        self.storage.chunks.get(&index)
    }

    /// Get a mutable chunk.
    ///
    /// **Notice**: This is not recommended as we may do something extra when you remove/set tiles.
    #[inline]
    pub fn get_chunk_mut(&mut self, index: IVec2) -> Option<&mut Vec<Option<Entity>>> {
        self.storage.chunks.get_mut(&index)
    }

    /// Set a tile.
    ///
    /// Overwrites the tile if it already exists.
    pub fn set(&mut self, commands: &mut Commands, index: IVec2, tile_builder: TileBuilder) {
        if let Some(previous) = self.storage.get_elem(index) {
            commands.entity(*previous).despawn();
        }
        let new_tile = tile_builder.build_component(index, &self, self.tilemap);

        let mut tile_entity = commands.spawn_empty();
        self.storage.set_elem(index, Some(tile_entity.id()));
        tile_entity.insert(new_tile);
    }

    #[inline]
    pub(crate) fn set_entity(&mut self, index: IVec2, entity: Option<Entity>) {
        self.storage.set_elem(index, entity);
    }

    /// Update some properties of a tile.
    #[inline]
    pub fn update(&mut self, commands: &mut Commands, index: IVec2, updater: TileUpdater) {
        if let Some(entity) = self.get(index) {
            commands.entity(entity).insert(updater);
        }
    }

    /// Remove a tile.
    #[inline]
    pub fn remove(&mut self, commands: &mut Commands, index: IVec2) {
        if let Some(entity) = self.get(index) {
            commands.entity(entity).insert(DespawnMe);
            self.set_entity(index, None);
        }
    }

    /// Remove the whole chunk and despawn all the tiles in it.
    #[inline]
    pub fn remove_chunk(&mut self, commands: &mut Commands, index: IVec2) {
        if let Some(chunk) = self.storage.remove_chunk(index) {
            chunk.into_iter().filter_map(|e| e).for_each(|e| {
                commands.entity(e).insert(DespawnMe);
            });
        }
    }

    /// Remove all the tiles in the tilemap.
    pub fn remove_all(&mut self, commands: &mut Commands) {
        self.storage
            .chunks
            .drain()
            .flat_map(|(_, chunk)| chunk.into_iter().filter_map(|e| e))
            .for_each(|entity| {
                commands.entity(entity).insert(DespawnMe);
            });
    }

    /// Despawn the entire tilemap.
    #[inline]
    pub fn despawn(&mut self, commands: &mut Commands) {
        self.remove_all(commands);
        commands.entity(self.tilemap).insert(DespawnMe);
    }

    /// Get the underlying storage and directly modify it.
    ///
    /// **Notice**: This may cause some problems if you do something inappropriately.
    #[inline]
    pub fn get_storage_raw(&mut self) -> &mut ChunkedStorage<Entity> {
        &mut self.storage
    }

    /// Fill a rectangle area with the same tile.
    pub fn fill_rect(
        &mut self,
        commands: &mut Commands,
        area: TileArea,
        tile_builder: TileBuilder,
    ) {
        let mut tile_batch = Vec::with_capacity(area.size());

        for y in area.origin.y..=area.dest.y {
            for x in area.origin.x..=area.dest.x {
                let index = IVec2 { x, y };
                self.remove(commands, index);
                let tile = tile_builder.build_component(index, &self, self.tilemap);
                let entity = if let Some(e) = self.get(index) {
                    e
                } else {
                    let e = commands.spawn_empty().id();
                    self.set_entity(index, Some(e));
                    e
                };
                tile_batch.push((entity, tile));
            }
        }

        commands.insert_or_spawn_batch(tile_batch);
    }

    /// Fill a rectangle area with tiles returned by `tile_builder`.
    ///
    /// Set `relative_index` to true if your function takes index relative to the area origin.
    pub fn fill_rect_custom(
        &mut self,
        commands: &mut Commands,
        area: TileArea,
        mut tile_builder: impl FnMut(IVec2) -> Option<TileBuilder>,
        relative_index: bool,
    ) {
        let mut tile_batch = Vec::with_capacity(area.size());

        for y in area.origin.y..=area.dest.y {
            for x in area.origin.x..=area.dest.x {
                let index = IVec2 { x, y };
                let Some(builder) = tile_builder({
                    if relative_index {
                        index - area.origin
                    } else {
                        index
                    }
                }) else {
                    continue;
                };

                self.remove(commands, index);
                let tile = builder.build_component(index, &self, self.tilemap);
                let entity = if let Some(e) = self.get(index) {
                    e
                } else {
                    let e = commands.spawn_empty().id();
                    self.set_entity(index, Some(e));
                    e
                };
                tile_batch.push((entity, tile));
            }
        }

        commands.insert_or_spawn_batch(tile_batch);
    }

    /// Fill a rectangle area with tiles from a buffer. This can be faster than setting them one by one.
    pub fn fill_with_buffer(
        &mut self,
        commands: &mut Commands,
        origin: IVec2,
        buffer: TileBuilderBuffer,
    ) {
        let batch = buffer
            .tiles
            .into_iter()
            .map(|(i, b)| {
                let tile = b.build_component(i + origin, &self, self.tilemap);

                if let Some(e) = self.get(tile.index) {
                    (e, tile)
                } else {
                    let e = commands.spawn_empty().id();
                    self.set_entity(tile.index, Some(e));
                    (e, tile)
                }
            })
            .collect::<Vec<_>>();

        commands.insert_or_spawn_batch(batch);
    }

    /// Simlar to `TilemapStorage::fill_rect()`.
    pub fn update_rect(&mut self, commands: &mut Commands, area: TileArea, updater: TileUpdater) {
        let mut batch = Vec::with_capacity(area.size());

        for y in area.origin.y..=area.dest.y {
            for x in area.origin.x..=area.dest.x {
                if let Some(entity) = self.get(IVec2 { x, y }) {
                    batch.push((entity, updater.clone()));
                }
            }
        }

        commands.insert_or_spawn_batch(batch);
    }

    /// Simlar to `TilemapStorage::fill_rect_custom()`.
    pub fn update_rect_custom(
        &mut self,
        commands: &mut Commands,
        area: TileArea,
        mut updater: impl FnMut(IVec2) -> TileUpdater,
        relative_index: bool,
    ) {
        let mut batch = Vec::with_capacity(area.size());

        for y in area.origin.y..=area.dest.y {
            for x in area.origin.x..=area.dest.x {
                if let Some(entity) = self.get(IVec2 { x, y }) {
                    batch.push((
                        entity,
                        updater(if relative_index {
                            IVec2 { x, y } - area.origin
                        } else {
                            IVec2 { x, y }
                        }),
                    ));
                }
            }
        }

        commands.insert_or_spawn_batch(batch);
    }
}

#[derive(Component, Default, Debug, Clone, Reflect)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub struct TilemapAnimations(pub(crate) Vec<i32>);

impl TilemapAnimations {
    /// Register a tile animation so you can use it in `TileBuilder::with_animation`.
    pub fn register_animation(&mut self, fps: u32, seq: Vec<u32>) -> TileAnimation {
        self.0.push(fps as i32);
        let start = self.0.len() as u32;
        let length = seq.len() as u32;
        self.0.extend(seq.into_iter().map(|i| i as i32));
        TileAnimation { start, length, fps }
    }
}

pub fn transform_syncer(
    mut tilemap_query: Query<(&TilemapTransform, &mut Transform), Changed<TilemapTransform>>,
) {
    tilemap_query.for_each_mut(|(tilemap_transform, mut transform)| {
        transform.translation = tilemap_transform
            .translation
            .extend(tilemap_transform.z_index as f32);
        transform.rotation = tilemap_transform.get_rotation_quat();
    });
}

pub fn queued_chunk_aabb_calculator(
    mut tilemaps_query: Query<(
        &mut TilemapStorage,
        &TilemapType,
        &TilePivot,
        &TilemapSlotSize,
        &TilemapTransform,
    )>,
) {
    tilemaps_query.par_iter_mut().for_each(
        |(mut storage, ty, tile_pivot, slot_size, transform)| {
            let chunk_size = storage.storage.chunk_size;
            let ext = storage
                .storage
                .calc_queue
                .drain()
                .map(|i| {
                    (
                        i,
                        Aabb2d::from_tilemap(
                            i,
                            chunk_size,
                            *ty,
                            tile_pivot.0,
                            slot_size.0,
                            *transform,
                        ),
                    )
                })
                .collect::<Vec<_>>();
            storage.storage.reserved.extend(ext);
        },
    );
}

pub fn tilemap_aabb_calculator(
    commands: ParallelCommands,
    mut tilemaps_query: Query<
        (
            &TilemapStorage,
            &TilemapType,
            &TilePivot,
            &TilemapSlotSize,
            &TilemapTransform,
        ),
        Changed<TilemapStorage>,
    >,
) {
    tilemaps_query
        .par_iter_mut()
        .for_each(|(storage, ty, tile_pivot, slot_size, transform)| {
            let mut chunk_aabb: Option<IAabb2d> = None;
            storage.storage.chunks.keys().for_each(|chunk_index| {
                if let Some(aabb) = &mut chunk_aabb {
                    aabb.expand_to_contain(*chunk_index);
                } else {
                    chunk_aabb = Some(IAabb2d::splat(*chunk_index));
                }
            });

            let Some(chunk_aabb) = chunk_aabb else {
                return;
            };

            let world_max = Aabb2d::from_tilemap(
                chunk_aabb.max,
                storage.storage.chunk_size,
                *ty,
                tile_pivot.0,
                slot_size.0,
                *transform,
            );
            let world_min = Aabb2d::from_tilemap(
                chunk_aabb.min,
                storage.storage.chunk_size,
                *ty,
                tile_pivot.0,
                slot_size.0,
                *transform,
            );
            let world_aabb = Aabb2d {
                min: world_min.min,
                max: world_max.max,
            };

            commands.command_scope(|mut c| {
                c.entity(storage.tilemap).insert(TilemapAabbs {
                    chunk_aabb,
                    world_aabb,
                });
            });
        });
}
