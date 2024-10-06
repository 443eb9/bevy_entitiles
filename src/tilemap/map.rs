use std::{f32::consts::SQRT_2, fmt::Debug};

use bevy::{
    asset::{Asset, Handle},
    ecs::{
        component::Component,
        query::Changed,
        system::{Query, SystemParamItem},
    },
    math::{IRect, Mat2, Quat, Rect, URect, Vec4},
    prelude::{Commands, Entity, IVec2, Image, UVec2, Vec2},
    reflect::Reflect,
    render::{
        render_asset::{PrepareAssetError, RenderAsset},
        render_resource::FilterMode,
    },
    sprite::TextureAtlasLayout,
    transform::components::Transform,
    utils::{HashMap, HashSet},
};

use crate::{
    math::{ext::RectFromTilemap, TileArea},
    tilemap::{
        buffers::TileBuilderBuffer,
        chunking::storage::{ChunkedStorage, EntityChunkedStorage},
        despawn::DespawnMe,
        tile::{RawTileAnimation, TileAnimation, TileBuilder, TileUpdater},
    },
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

/// Actually four directions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub enum TilemapRotation {
    #[default]
    None = 0,
    Cw90 = 90,
    Cw180 = 180,
    Cw270 = 270,
}

/// A tilemap transform. Using the `Transform` component is meaningless.
#[derive(Component, Debug, Clone, Copy, Default, Reflect)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub struct TilemapTransform {
    pub translation: Vec2,
    pub z_index: f32,
    pub rotation: TilemapRotation,
}

impl TilemapTransform {
    /// The transform with no translation and rotation.
    pub const IDENTITY: Self = Self {
        translation: Vec2::ZERO,
        z_index: 0.,
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
    pub fn from_translation_3d(translation: Vec2, z: f32) -> Self {
        Self {
            translation,
            z_index: z,
            ..Default::default()
        }
    }

    #[inline]
    pub fn from_z_index(z: f32) -> Self {
        Self {
            z_index: z,
            ..Default::default()
        }
    }

    #[inline]
    pub fn transform_point(&self, point: Vec2) -> Vec2 {
        self.apply_translation(self.apply_rotation(point))
    }

    pub fn transform_rect(&self, aabb: Rect) -> Rect {
        let min = self.transform_point(aabb.min);
        let max = self.transform_point(aabb.max);

        match self.rotation {
            TilemapRotation::None => Rect { min, max },
            TilemapRotation::Cw90 => Rect::new(max.x, min.y, min.x, max.y),
            TilemapRotation::Cw180 => Rect::new(max.x, max.y, min.x, min.y),
            TilemapRotation::Cw270 => Rect::new(min.x, max.y, max.x, min.y),
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

bitflags::bitflags! {
    /// Flip the tilemap along the x or y axis.
    #[derive(Component, Debug, Clone, Copy)]
    pub struct TilemapAxisFlip: u32 {
        const NONE = 0b00;
        const X    = 0b01;
        const Y    = 0b10;
    }
}

impl Default for TilemapAxisFlip {
    fn default() -> Self {
        Self::NONE
    }
}

impl TilemapAxisFlip {
    /// Get the flip as a `Vec2` where `1` means no flip and `-1` means flip.
    pub fn as_vec2(self) -> Vec2 {
        let mut v = Vec2::ONE;
        if self.contains(Self::X) {
            v.x = -1.;
        }
        if self.contains(Self::Y) {
            v.y = -1.;
        }
        v
    }
}

#[derive(Asset, Clone, Default, Debug, Reflect)]
pub struct TilemapTextures {
    pub(crate) textures: Vec<TilemapTexture>,
    pub(crate) start_index: Vec<u32>,
    pub(crate) uv_scales: Vec<Vec2>,
    pub(crate) max_size: UVec2,
    #[reflect(ignore)]
    pub(crate) filter_mode: FilterMode,
}

impl RenderAsset for TilemapTextures {
    type SourceAsset = Self;

    type Param = ();

    fn prepare_asset(
        source_asset: Self::SourceAsset,
        _param: &mut SystemParamItem<Self::Param>,
    ) -> Result<Self, PrepareAssetError<Self::SourceAsset>> {
        Ok(source_asset)
    }
}

impl TilemapTextures {
    pub fn single(texture: TilemapTexture, filter_mode: FilterMode) -> Self {
        Self::new(vec![texture], filter_mode)
    }

    pub fn new(textures: Vec<TilemapTexture>, filter_mode: FilterMode) -> Self {
        let mut start_index = Vec::with_capacity(textures.len());
        let mut cur = 0;
        let mut max_size = UVec2::ZERO;

        for tex in &textures {
            start_index.push(cur);
            cur += tex.tile_count();
            max_size = max_size.max(tex.desc.size);
        }

        Self {
            uv_scales: textures
                .iter()
                .map(|t| t.desc.size.as_vec2() / max_size.as_vec2())
                .collect(),
            textures,
            start_index,
            max_size,
            filter_mode,
        }
    }

    pub fn assert_uniform_tile_size(&self) {
        if self.textures.is_empty() {
            return;
        }

        for t in &self.textures {
            assert_eq!(t.desc.tile_size, self.textures[0].desc.tile_size);
        }
    }

    pub fn total_tile_count(&self) -> u32 {
        self.textures
            .iter()
            .fold(0, |acc, tex| acc + tex.tile_count())
    }

    pub fn iter_packed(&self) -> impl Iterator<Item = (&TilemapTexture, u32)> {
        self.textures.iter().zip(self.start_index.iter().cloned())
    }
}

/// A tilemap texture. It's similar to `TextureAtlas`.
#[derive(Clone, Default, Debug, Reflect)]
pub struct TilemapTexture {
    pub(crate) texture: Handle<Image>,
    pub(crate) desc: TilemapTextureDescriptor,
}

impl TilemapTexture {
    pub fn new(texture: Handle<Image>, desc: TilemapTextureDescriptor) -> Self {
        Self { texture, desc }
    }

    #[inline]
    pub fn clone_weak(&self) -> Handle<Image> {
        self.texture.clone_weak()
    }

    #[inline]
    pub fn desc(&self) -> &TilemapTextureDescriptor {
        &self.desc
    }

    #[inline]
    pub fn handle(&self) -> &Handle<Image> {
        &self.texture
    }

    #[inline]
    pub fn tile_count(&self) -> u32 {
        let t = self.desc.size / self.desc.tile_size;
        t.x * t.y
    }

    pub fn as_atlas_layout(&self) -> TextureAtlasLayout {
        TextureAtlasLayout::from_grid(
            self.desc.tile_size,
            self.desc.size.x,
            self.desc.size.y,
            Some(UVec2::ZERO),
            Some(UVec2::ZERO),
        )
    }

    /// Get the atlas rect of a tile in uv coordinates.
    pub fn get_atlas_rect(&self, index: u32) -> Rect {
        let tile_count = self.desc.size / self.desc.tile_size;
        let tile_index = Vec2::new((index % tile_count.x) as f32, (index / tile_count.x) as f32);
        let tile_size = self.desc.tile_size.as_vec2() / self.desc.size.as_vec2();
        Rect {
            min: tile_index * tile_size,
            max: (tile_index + Vec2::ONE) * tile_size,
        }
    }

    /// Get the atlas rect of a tile in pixel coordinates.
    pub fn get_atlas_urect(&self, index: u32) -> URect {
        let tile_count = self.desc.size / self.desc.tile_size;
        let tile_index = UVec2::new(index % tile_count.x, index / tile_count.x);
        URect {
            min: tile_index * self.desc.tile_size,
            max: (tile_index + 1) * self.desc.tile_size - 1,
        }
    }
}

#[derive(Component, Debug, Default, Clone)]
pub struct WaitForTextureUsageChange;

/// A descriptor for a tilemap texture.
#[derive(Clone, Copy, Default, Debug, PartialEq, Reflect)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub struct TilemapTextureDescriptor {
    pub(crate) size: UVec2,
    pub(crate) tile_size: UVec2,
}

impl TilemapTextureDescriptor {
    pub fn new(size: UVec2, tile_size: UVec2) -> Self {
        assert_eq!(
            size % tile_size,
            UVec2::ZERO,
            "Invalid tilemap texture descriptor! The size must be divisible by the tile size!"
        );

        Self { size, tile_size }
    }
}

#[derive(Component, Default, Debug, Clone, Reflect)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub struct TilemapName(pub String);

/// The actual rendered size of each tile mesh in pixels.
///
/// Every tile is acutally a square mesh like this:
/// ```text
/// +----+
/// |    | ← y
/// |    |
/// +----+
///  ↑x
/// ````
/// and the texture atlas will be rendered on it.
#[derive(Component, Default, Debug, Clone, Copy, Reflect)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub struct TileRenderSize(pub Vec2);

/// The gap between each tile mesh in pixels.
///
/// The tilemap mesh actually looks like this:
/// ```text
/// +----+----+----+
/// |    |    |    |
/// |    |    |    |
/// +----+----+----+
/// |    |    |    |
/// |    |    |    | ← y
/// +----+----+----+
///             ↑x
/// ```
/// You can use this to make margins or paddings between tiles.
#[derive(Component, Default, Debug, Clone, Copy, Reflect)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub struct TilemapSlotSize(pub Vec2);

/// The pivot of each tile mesh.
///
/// Every tile is acutally a square mesh like this:
/// ```text
///                 +----+
///                 |    |
///                 |    |
/// default pivot → +----+
/// (0., 0.)
/// ````
/// Changing this will affect the tile's scale ratio and it's position.
#[derive(Component, Default, Debug, Clone, Copy, Reflect)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub struct TilePivot(pub Vec2);

/// The opacity of each tile layer.
#[derive(Component, Debug, Clone, Copy, Reflect)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub struct TilemapLayerOpacities(pub Vec4);

impl Default for TilemapLayerOpacities {
    fn default() -> Self {
        Self(Vec4::ONE)
    }
}

/// The tilemap's aabb.
#[derive(Component, Default, Debug, Clone, Copy, Reflect)]
pub struct TilemapAabbs {
    pub(crate) chunk_aabb: IRect,
    pub(crate) world_aabb: Rect,
}

impl TilemapAabbs {
    /// The aabb of the whole tilemap in chunk coordinates.
    pub fn chunk_rect(&self) -> IRect {
        self.chunk_aabb
    }

    /// The aabb of the whole tilemap in world coordinates.
    pub fn world_rect(&self) -> Rect {
        self.world_aabb
    }
}

/// The tilemap's storage. It stores all the tiles in entity form.
#[derive(Component, Debug, Clone, Reflect)]
pub struct TilemapStorage {
    pub(crate) tilemap: Entity,
    pub(crate) storage: EntityChunkedStorage,
    pub(crate) reserved: HashMap<IVec2, Rect>,
    pub(crate) calc_queue: HashSet<IVec2>,
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
            reserved: Default::default(),
            calc_queue: Default::default(),
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
        self.storage.set_elem(index, tile_entity.id());
        self.reserve(new_tile.chunk_index);
        tile_entity.insert(new_tile);
    }

    #[inline]
    pub(crate) fn set_entity(&mut self, index: IVec2, entity: Option<Entity>) {
        if let Some(e) = entity {
            let (chunk_index, in_chunk_index) = self.storage.transform_index(index);
            self.storage
                .set_elem_precise(chunk_index, in_chunk_index, e);
            self.reserve(chunk_index);
        } else {
            self.storage.remove_elem(index);
        }
    }

    #[inline]
    #[allow(dead_code)]
    pub(crate) fn set_chunk_entity(&mut self, index: IVec2, chunk: Vec<Option<Entity>>) {
        self.storage.chunks.insert(index, chunk);
        self.reserve(index);
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

    /// Declare that a chunk is existent.
    ///
    /// Use `reserve_with_aabb` if you can provide the aabb.
    /// Or `reserve_many` to reserve multiple chunks.
    #[inline]
    pub fn reserve(&mut self, index: IVec2) {
        self.queue_aabb(index);
    }

    /// `reserve()` the chunk at `index` with the known `aabb`.
    #[inline]
    pub fn reserve_with_rect(&mut self, index: IVec2, aabb: Rect) {
        self.reserved.insert(index, aabb);
    }

    /// `reserve()` all the chunks in the iterator.
    #[inline]
    pub fn reserve_many(&mut self, indices: impl Iterator<Item = IVec2>) {
        indices.for_each(|i| {
            self.queue_aabb(i);
        });
    }

    /// `reserve_with_aabb()` all the chunks in the iterator with the known aabbs.
    #[inline]
    pub fn reserve_many_with_rects(&mut self, indices: impl Iterator<Item = (IVec2, Rect)>) {
        self.reserved.extend(indices);
    }

    #[inline]
    fn queue_aabb(&mut self, index: IVec2) {
        if !self.reserved.contains_key(&index) {
            self.calc_queue.insert(index);
        }
    }

    /// Despawn the entire tilemap.
    ///
    /// **Notice** this is the **only** and easiest way you can safely despawn the tilemap.
    #[inline]
    pub fn despawn(&mut self, commands: &mut Commands) {
        self.remove_all(commands);
        commands.entity(self.tilemap).insert(DespawnMe);
    }

    /// Get the underlying storage and directly modify it.
    ///
    /// **Notice**: This may cause some problems if you do something inappropriately.
    #[inline]
    pub fn get_storage_raw(&mut self) -> &mut EntityChunkedStorage {
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
                let tile = tile_builder.build_component(index, &self, self.tilemap);
                let entity = self.get(index).unwrap_or_else(|| {
                    let e = commands.spawn_empty().id();
                    self.set_entity(index, Some(e));
                    e
                });
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

                let tile = builder.build_component(index, &self, self.tilemap);
                let entity = self.get(index).unwrap_or_else(|| {
                    let e = commands.spawn_empty().id();
                    self.set_entity(index, Some(e));
                    e
                });
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

/// The tilemap's animation buffer.
///
/// Its format is `[fps, atlas_index_1, ..., atlas_index_n, fps, atlas_index_1, ..., atlas_index_n, ...]`.
///
/// If `atlas` feature is enabled, then the format is
///
/// `[fps, texture_index_1, atlas_index_1, ..., texture_index_n, atlas_index_n, fps, texture_index_1, atlas_index_1, ..., texture_index_n, atlas_index_n, ...]`
#[derive(Component, Debug, Clone, Reflect)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub struct TilemapAnimations(pub(crate) Vec<i32>);

impl Default for TilemapAnimations {
    fn default() -> Self {
        // Dummy data to ensure the binding resource can be created.
        Self(vec![0])
    }
}

impl TilemapAnimations {
    /// Register a tile animation so you can use it in `TileBuilder::with_animation`.
    pub fn register(&mut self, anim: RawTileAnimation) -> TileAnimation {
        self.0.push(anim.fps as i32);
        let start = self.0.len() as u32;
        let length = anim.sequence.len() as u32;

        #[cfg(not(feature = "atlas"))]
        self.0.extend(anim.sequence.into_iter().map(|i| i as i32));
        #[cfg(feature = "atlas")]
        self.0.extend(
            anim.sequence
                .into_iter()
                .flat_map(|(t, a)| [t as i32, a as i32]),
        );

        TileAnimation {
            start,
            length,
            fps: anim.fps,
        }
    }
}

pub fn transform_syncer(
    mut tilemap_query: Query<(&TilemapTransform, &mut Transform), Changed<TilemapTransform>>,
) {
    tilemap_query
        .iter_mut()
        .for_each(|(tilemap_transform, mut transform)| {
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
        &TilemapAxisFlip,
        &TilemapSlotSize,
        &TilemapTransform,
    )>,
) {
    tilemaps_query.par_iter_mut().for_each(
        |(mut storage, ty, tile_pivot, axis_direction, slot_size, transform)| {
            let chunk_size = storage.storage.chunk_size;
            let ext = storage
                .calc_queue
                .drain()
                .map(|i| {
                    (
                        i,
                        Rect::from_tilemap(
                            i,
                            chunk_size,
                            *ty,
                            tile_pivot.0,
                            *axis_direction,
                            slot_size.0,
                            *transform,
                        ),
                    )
                })
                .collect::<Vec<_>>();
            storage.reserved.extend(ext);
        },
    );
}

pub fn tilemap_aabb_calculator(
    mut tilemaps_query: Query<
        (
            &mut TilemapAabbs,
            &TilemapStorage,
            &TilemapType,
            &TilePivot,
            &TilemapAxisFlip,
            &TilemapSlotSize,
            &TilemapTransform,
        ),
        Changed<TilemapStorage>,
    >,
) {
    tilemaps_query.par_iter_mut().for_each(
        |(mut aabbs, storage, ty, tile_pivot, axis_direction, slot_size, transform)| {
            let mut chunk_aabb: Option<IRect> = None;
            storage.storage.chunks.keys().for_each(|chunk_index| {
                if let Some(aabb) = &mut chunk_aabb {
                    *aabb = aabb.union_point(*chunk_index);
                } else {
                    chunk_aabb = Some(IRect::from_center_size(*chunk_index, IVec2::ZERO));
                }
            });

            let Some(chunk_aabb) = chunk_aabb else {
                return;
            };

            let world_max = Rect::from_tilemap(
                chunk_aabb.max,
                storage.storage.chunk_size,
                *ty,
                tile_pivot.0,
                *axis_direction,
                slot_size.0,
                *transform,
            );
            let world_min = Rect::from_tilemap(
                chunk_aabb.min,
                storage.storage.chunk_size,
                *ty,
                tile_pivot.0,
                *axis_direction,
                slot_size.0,
                *transform,
            );

            aabbs.chunk_aabb = chunk_aabb;
            aabbs.world_aabb = Rect {
                min: world_min.min,
                max: world_max.max,
            };
        },
    );
}
