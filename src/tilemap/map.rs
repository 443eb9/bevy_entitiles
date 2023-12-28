use bevy::{
    ecs::component::Component,
    hierarchy::DespawnRecursiveExt,
    math::{Mat2, Vec4},
    prelude::{Assets, Commands, Entity, IVec2, Image, ResMut, UVec2, Vec2},
    reflect::Reflect,
    render::render_resource::TextureUsages,
};

use crate::{
    math::{aabb::AabbBox2d, FillArea},
    render::{buffer::TileAnimation, texture::TilemapTexture},
    MAX_ANIM_COUNT, MAX_LAYER_COUNT,
};

use super::{
    layer::{LayerInserter, LayerUpdater},
    tile::{TileBuilder, TileType},
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

    pub fn transform_aabb(&self, aabb: AabbBox2d) -> AabbBox2d {
        let min = self.transform_point(aabb.min);
        let max = self.transform_point(aabb.max);

        match self.rotation {
            TilemapRotation::None => AabbBox2d { min, max },
            TilemapRotation::Cw90 => AabbBox2d::new(max.x, min.y, min.x, max.y),
            TilemapRotation::Cw180 => AabbBox2d::new(max.x, max.y, min.x, min.y),
            TilemapRotation::Cw270 => AabbBox2d::new(min.x, max.y, max.x, min.y),
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

pub struct TilemapBuilder {
    pub name: String,
    pub tile_type: TileType,
    pub ext_dir: Vec2,
    pub size: UVec2,
    pub tile_render_size: Vec2,
    pub tile_slot_size: Vec2,
    pub pivot: Vec2,
    pub render_chunk_size: u32,
    pub texture: Option<TilemapTexture>,
    pub transform: TilemapTransform,
    pub anim_seqs: [TileAnimation; MAX_ANIM_COUNT],
}

impl TilemapBuilder {
    /// Create a new tilemap builder.
    pub fn new(ty: TileType, size: UVec2, tile_render_size: Vec2, name: String) -> Self {
        Self {
            name,
            tile_type: ty,
            ext_dir: Vec2::ONE,
            size,
            tile_render_size,
            tile_slot_size: tile_render_size,
            pivot: Vec2::ZERO,
            texture: None,
            render_chunk_size: 32,
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

    /// Override render chunk size. Default is 32.
    pub fn with_render_chunk_size(&mut self, size: u32) -> &mut Self {
        self.render_chunk_size = size;
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
            size: self.size,
            tiles: vec![None; (self.size.x * self.size.y) as usize],
            render_chunk_size: self.render_chunk_size,
            tile_slot_size: self.tile_slot_size,
            pivot: self.pivot,
            texture: self.texture.clone(),
            layer_opacities: Vec4::ONE,
            aabb: AabbBox2d::from_tilemap_builder(&self),
            transform: self.transform,
            anim_seqs: self.anim_seqs,
            anim_counts: 0,
        };
        entity.insert((WaitForTextureUsageChange, tilemap.clone()));
        tilemap
    }
}

#[cfg(feature = "serializing")]
#[derive(serde::Serialize, serde::Deserialize, Clone, Reflect)]
pub struct TilemapPattern {
    pub size: UVec2,
    pub tiles: Vec<Option<crate::serializing::SerializedTile>>,
}

#[cfg(feature = "serializing")]
impl TilemapPattern {
    pub fn get(&self, index: UVec2) -> Option<&crate::serializing::SerializedTile> {
        self.tiles[(index.y * self.size.x + index.x) as usize].as_ref()
    }
}

#[derive(Component, Clone, Reflect)]
pub struct Tilemap {
    pub(crate) id: Entity,
    pub(crate) name: String,
    pub(crate) tile_type: TileType,
    pub(crate) ext_dir: Vec2,
    pub(crate) size: UVec2,
    pub(crate) tile_render_size: Vec2,
    pub(crate) tile_slot_size: Vec2,
    pub(crate) pivot: Vec2,
    pub(crate) render_chunk_size: u32,
    pub(crate) texture: Option<TilemapTexture>,
    pub(crate) layer_opacities: Vec4,
    pub(crate) tiles: Vec<Option<Entity>>,
    pub(crate) aabb: AabbBox2d,
    pub(crate) transform: TilemapTransform,
    pub(crate) anim_seqs: [TileAnimation; MAX_ANIM_COUNT],
    pub(crate) anim_counts: usize,
}

impl Tilemap {
    /// Get a tile.
    pub fn get(&self, index: UVec2) -> Option<Entity> {
        if self.is_out_of_tilemap_uvec(index) {
            return None;
        }

        self.get_unchecked(index)
    }

    pub(crate) fn get_unchecked(&self, index: UVec2) -> Option<Entity> {
        self.tiles[self.transform_index(index)]
    }

    /// Set a tile.
    ///
    /// Overwrites the tile if it already exists.
    pub fn set(&mut self, commands: &mut Commands, index: UVec2, tile_builder: TileBuilder) {
        if self.is_out_of_tilemap_uvec(index) {
            return;
        }

        self.set_unchecked(commands, index, tile_builder);
    }

    /// Overwrites all the tiles.
    pub fn set_all(&mut self, commands: &mut Commands, tiles: &Vec<Option<TileBuilder>>) {
        assert_eq!(
            tiles.len(),
            self.tiles.len(),
            "tiles length must be equal to the tilemap size"
        );

        for (i, tile) in tiles.iter().enumerate() {
            let index = UVec2 {
                x: i as u32 % self.size.x,
                y: i as u32 / self.size.x,
            };

            if let Some(t) = tile {
                self.set_unchecked(commands, index, t.clone());
            } else {
                self.remove(commands, index);
            }
        }
    }

    pub(crate) fn set_unchecked(
        &mut self,
        commands: &mut Commands,
        index: UVec2,
        tile_builder: TileBuilder,
    ) {
        let vec_idx = self.transform_index(index);
        if let Some(previous) = self.tiles[vec_idx] {
            commands.entity(previous).despawn();
        }
        let new_tile = tile_builder.build(commands, index, self);
        self.tiles[vec_idx] = Some(new_tile);
    }

    pub fn insert_layer(&mut self, commands: &mut Commands, index: UVec2, inserter: LayerInserter) {
        if let Some(tile) = self.get(index) {
            commands.entity(tile).insert(inserter);
        }
    }

    pub fn update(&mut self, commands: &mut Commands, index: UVec2, updater: LayerUpdater) {
        if self.get(index).is_none() {
            return;
        }

        self.update_unchecked(commands, index, updater);
    }

    #[inline]
    pub(crate) fn update_unchecked(
        &mut self,
        commands: &mut Commands,
        index: UVec2,
        updater: LayerUpdater,
    ) {
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
    pub fn remove(&mut self, commands: &mut Commands, index: UVec2) {
        if self.is_out_of_tilemap_uvec(index) || self.get(index).is_none() {
            return;
        }

        self.remove_unchecked(commands, index);
    }

    pub(crate) fn remove_unchecked(&mut self, commands: &mut Commands, index: UVec2) {
        let index = self.transform_index(index);
        commands.entity(self.tiles[index].unwrap()).despawn();
        self.tiles[index] = None;
    }

    /// Fill a rectangle area with tiles.
    pub fn fill_rect(
        &mut self,
        commands: &mut Commands,
        area: FillArea,
        tile_builder: TileBuilder,
    ) {
        let mut tile_batch = Vec::with_capacity(area.size());
        let mut anim_batch = Vec::with_capacity(area.size());

        for y in area.origin.y..=area.dest.y {
            for x in area.origin.x..=area.dest.x {
                self.remove(commands, UVec2 { x, y });
                let (tile, anim) = tile_builder.build_component(UVec2 { x, y }, self);
                tile_batch.push(tile);
                if let Some(anim) = anim {
                    anim_batch.push(anim);
                }
            }
        }

        commands.spawn_batch(tile_batch);
        commands.spawn_batch(anim_batch);
    }

    /// Fill a rectangle area with tiles returned by `tile_builder`.
    ///
    /// Set `relative_index` to true if your function takes index relative to the area origin.
    pub fn fill_rect_custom(
        &mut self,
        commands: &mut Commands,
        area: FillArea,
        mut tile_builder: impl FnMut(UVec2) -> TileBuilder,
        relative_index: bool,
    ) {
        let mut tile_batch = Vec::with_capacity(area.size());
        let mut anim_batch = Vec::with_capacity(area.size());

        for y in area.origin.y..=area.dest.y {
            for x in area.origin.x..=area.dest.x {
                self.remove(commands, UVec2 { x, y });
                let builder = tile_builder(if relative_index {
                    UVec2::new(x, y) - area.origin
                } else {
                    UVec2::new(x, y)
                });
                let (tile, anim) = builder.build_component(UVec2 { x, y }, self);
                tile_batch.push(tile);
                if let Some(anim) = anim {
                    anim_batch.push(anim);
                }
            }
        }

        commands.spawn_batch(tile_batch);
        commands.spawn_batch(anim_batch);
    }

    // TODO implement this
    // #[cfg(feature = "serializing")]
    // pub fn set_pattern(&mut self, commands: &mut Commands, pattern: TilemapPattern, origin: UVec2) {
    //     for y in 0..pattern.size.y {
    //         for x in 0..pattern.size.x {
    //             let index = UVec2 { x, y };
    //             if let Some(tile) = pattern.get(index) {
    //                 self.set(
    //                     commands,
    //                     index + origin,
    //                     &TileBuilder::from_serialized_tile(tile),
    //                 );
    //             }
    //         }
    //     }
    // }

    /// Simlar to `Tilemap::fill_rect()`.
    pub fn update_rect(&mut self, commands: &mut Commands, area: FillArea, updater: LayerUpdater) {
        let mut batch = Vec::with_capacity(area.size());

        for y in area.origin.y..=area.dest.y {
            for x in area.origin.x..=area.dest.x {
                if let Some(entity) = self.get(UVec2 { x, y }) {
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
        area: FillArea,
        mut updater: impl FnMut(UVec2) -> LayerUpdater,
        relative_index: bool,
    ) {
        let mut batch = Vec::with_capacity(area.size());

        for y in area.origin.y..=area.dest.y {
            for x in area.origin.x..=area.dest.x {
                if let Some(entity) = self.get(UVec2 { x, y }) {
                    batch.push((
                        entity,
                        updater(if relative_index {
                            UVec2 { x, y } - area.origin
                        } else {
                            UVec2 { x, y }
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
    pub fn register_animation(&mut self, anim: TileAnimation) -> usize {
        assert!(
            self.anim_counts + 1 < MAX_ANIM_COUNT,
            "too many animations!, max is {}",
            MAX_ANIM_COUNT
        );

        let index = self.anim_counts;
        self.anim_seqs[index] = anim;
        self.anim_counts += 1;
        index
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
    #[inline]
    pub fn index_to_world(&self, index: UVec2) -> Vec2 {
        self.index_inf_to_world(index.as_ivec2())
    }

    /// Get the world position of the center of a slot.
    ///
    /// This method does not limit the index to the tilemap size.
    #[inline]
    pub fn index_inf_to_world(&self, index: IVec2) -> Vec2 {
        let index = index.as_vec2();
        match self.tile_type {
            TileType::Square => self
                .transform
                .transform_point((index - self.pivot) * self.tile_slot_size),
            TileType::Isometric => self.transform.transform_point(
                (Vec2 {
                    x: (index.x - index.y - 1.),
                    y: (index.x + index.y),
                } / 2.
                    - self.pivot)
                    * self.tile_slot_size,
            ),
            TileType::Hexagonal(legs) => self.transform.transform_point(Vec2 {
                x: self.tile_slot_size.x * (index.x - 0.5 * index.y - self.pivot.x),
                y: (self.tile_slot_size.y + legs as f32) / 2. * (index.y - self.pivot.y),
            }),
        }
    }

    #[inline]
    pub fn transform_index(&self, index: UVec2) -> usize {
        (index.y * self.size.x + index.x) as usize
    }

    #[inline]
    pub fn is_out_of_tilemap_uvec(&self, index: UVec2) -> bool {
        index.x >= self.size.x || index.y >= self.size.y
    }

    #[inline]
    pub fn is_out_of_tilemap_ivec(&self, index: IVec2) -> bool {
        index.x < 0 || index.y < 0 || index.x >= self.size.x as i32 || index.y >= self.size.y as i32
    }

    #[inline]
    pub fn get_tile_convex_hull(&self, index: UVec2) -> Vec<Vec2> {
        let offset = self.index_to_world(index);
        let (x, y) = (self.tile_slot_size.x, self.tile_slot_size.y);
        let res = match self.tile_type {
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
        };

        res.into_iter()
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
