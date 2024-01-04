use std::fmt::Debug;

use bevy::{
    ecs::component::Component,
    hierarchy::{BuildChildren, DespawnRecursiveExt},
    math::{Mat2, Quat, Vec4},
    prelude::{Assets, Commands, Entity, IVec2, Image, ResMut, SpatialBundle, UVec2, Vec2},
    reflect::Reflect,
    render::render_resource::TextureUsages,
    transform::components::Transform,
    utils::HashMap,
};

use crate::{
    math::{aabb::Aabb2d, extension::DivToFloor, TileArea},
    render::{buffer::TileAnimation, texture::TilemapTexture},
    MAX_ANIM_COUNT, MAX_LAYER_COUNT,
};

use super::{
    layer::TileUpdater,
    tile::{TileBuffer, TileBuilder, TileType},
};

#[derive(Debug, Clone, Copy, Default, Reflect)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub enum TilemapRotation {
    #[default]
    None = 0,
    Cw90 = 90,
    Cw180 = 180,
    Cw270 = 270,
}

#[derive(Debug, Clone, Copy, Default, Reflect)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub struct TilemapTransform {
    pub translation: Vec2,
    pub z_index: i32,
    pub rotation: TilemapRotation,
}

impl TilemapTransform {
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

pub struct TilemapBuilder {
    pub(crate) name: String,
    pub(crate) tile_type: TileType,
    pub(crate) ext_dir: Vec2,
    pub(crate) tile_render_size: Vec2,
    pub(crate) tile_slot_size: Vec2,
    pub(crate) pivot: Vec2,
    pub(crate) chunk_size: u32,
    pub(crate) texture: Option<TilemapTexture>,
    pub(crate) transform: TilemapTransform,
    pub(crate) anim_seqs: [TileAnimation; MAX_ANIM_COUNT],
}

impl TilemapBuilder {
    /// Create a new tilemap builder.
    pub fn new(ty: TileType, tile_render_size: Vec2, name: String) -> Self {
        Self {
            name,
            tile_type: ty,
            ext_dir: Vec2::ONE,
            tile_render_size,
            tile_slot_size: tile_render_size,
            pivot: Vec2::ZERO,
            texture: None,
            chunk_size: 32,
            transform: TilemapTransform::default(),
            anim_seqs: [TileAnimation::default(); MAX_ANIM_COUNT],
        }
    }

    /// Override z index. Default is 0.
    /// The higher the value of z_index, the less likely to be covered.
    pub fn with_z_index(&mut self, z_index: i32) -> &mut Self {
        self.transform.z_index = z_index;
        self
    }

    /// Override chunk size. Default is 32.
    pub fn with_chunk_size(&mut self, size: u32) -> &mut Self {
        self.chunk_size = size;
        self
    }

    /// Override translation. Default is `Vec2::ZERO`.
    pub fn with_translation(&mut self, translation: Vec2) -> &mut Self {
        self.transform.translation = translation;
        self
    }

    /// Override rotation. Default is `TilemapRotation::None`.
    pub fn with_rotation(&mut self, rotation: TilemapRotation) -> &mut Self {
        self.transform.rotation = rotation;
        self
    }

    /// Assign a texture to the tilemap.
    pub fn with_texture(&mut self, texture: TilemapTexture) -> &mut Self {
        self.texture = Some(texture);
        self
    }

    /// Override the size of the tilemap slots. Default is equal to `tile_render_size`.
    pub fn with_tile_slot_size(&mut self, tile_slot_size: Vec2) -> &mut Self {
        self.tile_slot_size = tile_slot_size;
        self
    }

    /// Override the pivot of the tile. Default is `[0., 0.]`.
    pub fn with_pivot(&mut self, pivot: Vec2) -> &mut Self {
        assert!(
            pivot.x >= 0. && pivot.x <= 1. && pivot.y >= 0. && pivot.y <= 1.,
            "pivot must be in range [0, 1]"
        );
        self.pivot = pivot;
        self
    }

    /// Override the extend direction of the tilemap. Default is `Vec2::ONE`.
    ///
    /// You can set this to `[1, -1]` or something to change the direction of the tilemap.
    pub fn with_extend_direction(&mut self, direction: Vec2) -> &mut Self {
        self.ext_dir = direction;
        self
    }

    /// Build the tilemap and spawn it into the world.
    /// You can modify the component and insert it back.
    pub fn build(&self, commands: &mut Commands) -> Tilemap {
        let mut entity = commands.spawn_empty();
        let tilemap = Tilemap {
            id: entity.id(),
            name: self.name.clone(),
            tile_render_size: self.tile_render_size,
            tile_type: self.tile_type,
            ext_dir: self.ext_dir,
            storage: TilemapStorage::new(self.chunk_size),
            chunk_size: self.chunk_size,
            tile_slot_size: self.tile_slot_size,
            pivot: self.pivot,
            texture: self.texture.clone(),
            layer_opacities: Vec4::ONE,
            // aabb: Aabb2d::from_tilemap_builder(&self),
            transform: self.transform,
            anim_seqs: self.anim_seqs,
            anim_counts: 0,
        };
        entity.insert((
            WaitForTextureUsageChange,
            tilemap.clone(),
            SpatialBundle {
                transform: tilemap.transform.into(),
                ..Default::default()
            },
        ));
        tilemap
    }
}

#[derive(Debug, Clone, Reflect)]
pub struct TilemapStorage<T: Debug + Clone + Reflect> {
    pub chunk_size: UVec2,
    pub chunks: HashMap<IVec2, Vec<Option<T>>>,
    pub down_left: IVec2,
    pub up_right: IVec2,
}

impl<T: Debug + Clone + Reflect> TilemapStorage<T> {
    pub fn new(chunk_size: u32) -> Self {
        Self {
            chunk_size: UVec2::splat(chunk_size),
            chunks: HashMap::new(),
            down_left: IVec2::ZERO,
            up_right: IVec2::ZERO,
        }
    }

    pub fn get(&self, index: IVec2) -> Option<&T> {
        let idx = self.transform_index(index);
        self.chunks
            .get(&idx.0)
            .and_then(|c| c.get(idx.1).as_ref().cloned())
            .and_then(|t| t.as_ref())
    }

    pub fn get_mut(&mut self, index: IVec2) -> Option<&mut T> {
        let idx = self.transform_index(index);
        if let Some(chunk) = self.chunks.get_mut(&idx.0) {
            chunk.get_mut(idx.1).map(|t| t.as_mut()).flatten()
        } else {
            None
        }
    }

    pub fn set(&mut self, index: IVec2, elem: Option<T>) {
        let idx = self.transform_index(index);
        self.chunks
            .entry(idx.0)
            .or_insert_with(|| vec![None; (self.chunk_size.x * self.chunk_size.y) as usize])
            [idx.1] = elem;

        self.down_left = self.down_left.min(index);
        self.up_right = self.up_right.max(index);
    }

    pub fn transform_index(&self, index: IVec2) -> (IVec2, usize) {
        let isize = self.chunk_size.as_ivec2();
        let c = index.div_to_floor(isize);
        let idx = index - c * isize;
        (c, (idx.y * isize.x + idx.x) as usize)
    }

    #[inline]
    pub fn size(&self) -> UVec2 {
        (self.up_right - self.down_left + IVec2::ONE).as_uvec2()
    }

    #[inline]
    pub fn usize(&self) -> usize {
        let size = self.size();
        (size.x * size.y) as usize
    }
}

#[derive(Component, Clone, Reflect)]
pub struct Tilemap {
    pub(crate) id: Entity,
    pub(crate) name: String,
    pub(crate) tile_type: TileType,
    pub(crate) ext_dir: Vec2,
    pub(crate) tile_render_size: Vec2,
    pub(crate) tile_slot_size: Vec2,
    pub(crate) pivot: Vec2,
    pub(crate) chunk_size: u32,
    pub(crate) texture: Option<TilemapTexture>,
    pub(crate) layer_opacities: Vec4,
    pub(crate) storage: TilemapStorage<Entity>,
    // pub(crate) aabb: Aabb2d,
    pub(crate) transform: TilemapTransform,
    pub(crate) anim_seqs: [TileAnimation; MAX_ANIM_COUNT],
    pub(crate) anim_counts: usize,
}

impl Tilemap {
    /// Get a tile.
    #[inline]
    pub fn get(&self, index: IVec2) -> Option<Entity> {
        self.storage.get(index).cloned()
    }

    /// Set a tile.
    ///
    /// Overwrites the tile if it already exists.
    pub fn set(&mut self, commands: &mut Commands, index: IVec2, tile_builder: TileBuilder) {
        if let Some(previous) = self.storage.get(index) {
            commands.entity(previous.clone()).despawn();
        }
        let new_tile = tile_builder.build(commands, index, self);
        self.storage.set(index, Some(new_tile));
    }

    #[inline]
    pub(crate) fn set_entity(&mut self, index: IVec2, entity: Entity) {
        self.storage.set(index, Some(entity));
    }

    pub fn update(&mut self, commands: &mut Commands, index: IVec2, updater: TileUpdater) {
        if self.get(index).is_none() {
            return;
        }

        commands.entity(self.get(index).unwrap()).insert(updater);
    }

    /// Set the opacity of a layer. Default is 1 for each layer.
    pub fn set_layer_opacity(&mut self, layer: usize, opacity: f32) -> &mut Self {
        assert!(
            layer <= MAX_LAYER_COUNT,
            "Currently we only support up to 4 layers!"
        );

        self.layer_opacities[layer] = opacity;
        self
    }

    /// Remove a tile.
    pub fn remove(&mut self, commands: &mut Commands, index: IVec2) {
        if self.get(index).is_none() {
            return;
        }

        if let Some(entity) = self.get(index) {
            commands.entity(entity).despawn_recursive();
            self.set_entity(index, entity);
        }
    }

    /// Fill a rectangle area with tiles.
    pub fn fill_rect(
        &mut self,
        commands: &mut Commands,
        area: TileArea,
        tile_builder: TileBuilder,
    ) {
        let mut tile_batch = Vec::with_capacity(area.size());
        let mut entities = Vec::with_capacity(area.size());

        for y in area.origin.y..=area.dest.y {
            for x in area.origin.x..=area.dest.x {
                let index = IVec2 { x, y };
                self.remove(commands, index);
                let tile = tile_builder.build_component(index, self);
                let entity = if let Some(e) = self.get(index) {
                    e
                } else {
                    let e = commands.spawn_empty().id();
                    self.set_entity(index, e);
                    entities.push(e);
                    e
                };
                tile_batch.push((entity, tile));
            }
        }

        commands.insert_or_spawn_batch(tile_batch);
        commands.entity(self.id).push_children(&entities);
    }

    /// Fill a rectangle area with tiles returned by `tile_builder`.
    ///
    /// Set `relative_index` to true if your function takes index relative to the area origin.
    pub fn fill_rect_custom(
        &mut self,
        commands: &mut Commands,
        area: TileArea,
        mut tile_builder: impl FnMut(IVec2) -> TileBuilder,
        relative_index: bool,
    ) {
        let mut tile_batch = Vec::with_capacity(area.size());
        let mut entities = Vec::with_capacity(area.size());

        for y in area.origin.y..=area.dest.y {
            for x in area.origin.x..=area.dest.x {
                let index = IVec2 { x, y };
                self.remove(commands, index);
                let builder = tile_builder(if relative_index {
                    index - area.origin
                } else {
                    index
                });

                let tile = builder.build_component(index, self);
                let entity = if let Some(e) = self.get(index) {
                    e
                } else {
                    let e = commands.spawn_empty().id();
                    self.set_entity(index, e);
                    entities.push(e);
                    e
                };
                tile_batch.push((entity, tile));
            }
        }

        commands.insert_or_spawn_batch(tile_batch);
        commands.entity(self.id).push_children(&entities);
    }

    /// Fill a rectangle area with tiles from a buffer. This can be faster than set them one by one.
    pub fn fill_with_buffer(&mut self, commands: &mut Commands, origin: IVec2, buffer: TileBuffer) {
        let mut entities = Vec::with_capacity((buffer.size.x * buffer.size.y) as usize);

        let batch = buffer
            .tiles
            .into_iter()
            .enumerate()
            .filter_map(|(index, b)| {
                if let Some(builder) = b {
                    let tile = builder.build_component(
                        UVec2 {
                            x: index as u32 % buffer.size.x,
                            y: index as u32 / buffer.size.x,
                        }
                        .as_ivec2()
                            + origin,
                        self,
                    );

                    if let Some(e) = self.get(tile.index) {
                        Some((e, tile))
                    } else {
                        let e = commands.spawn_empty().id();
                        self.set_entity(tile.index, e);
                        entities.push(e);
                        Some((e, tile))
                    }
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        commands.insert_or_spawn_batch(batch);
        commands.entity(self.id).push_children(&entities);
    }

    /// Simlar to `Tilemap::fill_rect()`.
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

    /// Simlar to `Tilemap::fill_rect_custom()`.
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

    /// Register a tile animation so you can use it in `TileBuilder::with_animation`.
    ///
    /// Returns the sequence index of the animation.
    pub fn register_animation(&mut self, anim: TileAnimation) -> u32 {
        assert!(
            self.anim_counts + 1 < MAX_ANIM_COUNT,
            "too many animations!, max is {}",
            MAX_ANIM_COUNT
        );

        let index = self.anim_counts;
        self.anim_seqs[index] = anim;
        self.anim_counts += 1;
        index as u32
    }

    /// Update a tile animation by overwriting the element at `index`.
    ///
    /// This does nothing if `index` is out of range.
    pub fn update_animation(&mut self, anim: TileAnimation, index: usize) {
        if index < MAX_ANIM_COUNT {
            self.anim_seqs[index] = anim;
        }
    }

    /// Remove the whole tilemap.
    pub fn delete(&mut self, commands: &mut Commands) {
        commands.entity(self.id).despawn_recursive();
    }

    /// Get the id of the tilemap.
    #[inline]
    pub fn id(&self) -> Entity {
        self.id
    }

    /// Get the name of the tilemap.
    #[inline]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    #[inline]
    pub fn transform(&self) -> TilemapTransform {
        self.transform
    }

    /// Get the world position of the center of a slot.
    pub fn index_to_world(&self, index: IVec2) -> Vec2 {
        let index = index.as_vec2();
        self.transform.transform_point({
            match self.tile_type {
                TileType::Square => (index - self.pivot) * self.tile_slot_size,
                TileType::Isometric => {
                    (Vec2 {
                        x: (index.x - index.y - 1.),
                        y: (index.x + index.y),
                    } / 2.
                        - self.pivot)
                        * self.tile_slot_size
                }
                TileType::Hexagonal(legs) => Vec2 {
                    x: self.tile_slot_size.x * (index.x - 0.5 * index.y - self.pivot.x),
                    y: (self.tile_slot_size.y + legs as f32) / 2. * (index.y - self.pivot.y),
                },
            }
        })
    }

    pub fn index_to_rel(&self, index: IVec2) -> Vec2 {
        self.index_to_world(index) - self.transform.translation
    }

    #[inline]
    pub fn transform_index(&self, index: IVec2) -> (IVec2, UVec2) {
        let c = index.div_to_floor(IVec2::splat(self.chunk_size as i32));
        (
            c,
            (index - c * IVec2::splat(self.chunk_size as i32)).as_uvec2(),
        )
    }

    pub fn get_tile_convex_hull(&self) -> Vec<Vec2> {
        let (x, y) = (self.tile_slot_size.x, self.tile_slot_size.y);
        match self.tile_type {
            TileType::Square => vec![
                Vec2 { x: 0., y: 0. },
                Vec2 { x: 0., y },
                Vec2 { x, y },
                Vec2 { x, y: 0. },
            ],
            TileType::Isometric => vec![
                Vec2 { x: 0., y: y / 2. },
                Vec2 { x: x / 2., y },
                Vec2 { x, y: y / 2. },
                Vec2 { x: x / 2., y: 0. },
            ],
            TileType::Hexagonal(c) => {
                let c = c as f32;
                let Vec2 { x: a, y: b } = self.tile_render_size;
                let half = (b - c) / 2.;

                vec![
                    Vec2 { x: 0., y: half },
                    Vec2 { x: 0., y: half + c },
                    Vec2 { x: a / 2., y: b },
                    Vec2 { x: a, y: half + c },
                    Vec2 { x: a, y: half },
                    Vec2 { x: a / 2., y: 0. },
                ]
            }
        }
    }

    pub fn get_tile_convex_hull_rel(&self, index: IVec2) -> Vec<Vec2> {
        let offset = self.index_to_rel(index);
        self.get_tile_convex_hull()
            .into_iter()
            .map(|p| self.transform.apply_rotation(p) + offset)
            .collect()
    }

    pub fn get_tile_convex_hull_world(&self, index: IVec2) -> Vec<Vec2> {
        let offset = self.index_to_world(index);
        self.get_tile_convex_hull()
            .into_iter()
            .map(|p| self.transform.apply_rotation(p) + offset)
            .collect()
    }

    /// Bevy doesn't set the `COPY_SRC` usage for images by default, so we need to do it manually.
    pub(crate) fn set_usage(
        &mut self,
        commands: &mut Commands,
        image_assets: &mut ResMut<Assets<Image>>,
    ) {
        let Some(texture) = &self.texture else {
            commands
                .entity(self.id)
                .remove::<WaitForTextureUsageChange>();
            return;
        };

        let Some(image) = image_assets.get(&texture.clone_weak()) else {
            return;
        };

        if !image
            .texture_descriptor
            .usage
            .contains(TextureUsages::COPY_SRC)
        {
            image_assets
                .get_mut(&texture.clone_weak())
                .unwrap()
                .texture_descriptor
                .usage
                .set(TextureUsages::COPY_SRC, true);
        }

        commands
            .entity(self.id)
            .remove::<WaitForTextureUsageChange>();
    }
}

#[derive(Component)]
pub struct WaitForTextureUsageChange;
