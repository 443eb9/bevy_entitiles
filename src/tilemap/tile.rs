use bevy::{
    ecs::system::{ParallelCommands, Query},
    hierarchy::BuildChildren,
    math::IVec2,
    prelude::{Commands, Component, Entity, UVec2, Vec4},
    reflect::Reflect,
    utils::HashMap,
};

use crate::math::aabb::IAabb2d;

use super::map::TilemapStorage;

#[derive(Debug, Default, Clone, Copy, Reflect)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub struct TileLayer {
    pub(crate) texture_index: i32,
    pub(crate) flip: u32,
}

impl TileLayer {
    pub fn new() -> Self {
        Self {
            texture_index: -1,
            flip: 0,
        }
    }

    pub fn with_texture_index(mut self, texture_index: u32) -> Self {
        self.texture_index = texture_index as i32;
        self
    }

    pub fn with_flip(mut self, flip: TileFlip) -> Self {
        self.flip |= flip as u32;
        self
    }

    #[allow(dead_code)]
    pub(crate) fn with_flip_raw(mut self, flip: u32) -> Self {
        self.flip = flip;
        self
    }
}

#[derive(Debug, Clone, Copy, Reflect)]
pub enum TileLayerPosition {
    Top,
    Bottom,
    Index(usize),
}

#[derive(Clone, Reflect)]
pub struct LayerUpdater {
    pub position: TileLayerPosition,
    pub layer: TileLayer,
}

#[derive(Default, Component, Clone, Reflect)]
pub struct TileUpdater {
    pub layer: Option<LayerUpdater>,
    pub color: Option<Vec4>,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, Reflect)]
pub enum TileFlip {
    None = 0b00,
    Horizontal = 0b01,
    Vertical = 0b10,
    Both = 0b11,
}

impl From<u32> for TileFlip {
    fn from(value: u32) -> Self {
        match value {
            0b00 => Self::None,
            0b01 => Self::Horizontal,
            0b10 => Self::Vertical,
            0b11 => Self::Both,
            _ => panic!("Invalid flip value! {}", value),
        }
    }
}

#[derive(Debug, Clone, Reflect)]
pub struct TileBuilder {
    pub(crate) texture: TileTexture,
    pub(crate) color: Vec4,
}

impl TileBuilder {
    /// Create a new tile builder.
    pub fn new() -> Self {
        Self {
            texture: TileTexture::Static(Vec::new()),
            color: Vec4::ONE,
        }
    }

    pub fn with_color(mut self, color: Vec4) -> Self {
        self.color = color;
        self
    }

    pub fn with_layer(mut self, index: usize, layer: TileLayer) -> Self {
        if let TileTexture::Static(ref mut tex) = self.texture {
            if tex.len() <= index {
                tex.resize(index + 1, TileLayer::new());
            }
            tex[index] = layer;
        }
        self
    }

    pub fn with_animation(mut self, animation: u32) -> Self {
        self.texture = TileTexture::Animated(animation);
        self
    }

    pub(crate) fn build(
        &self,
        commands: &mut Commands,
        index: IVec2,
        storage: &TilemapStorage,
        tilemap: Entity,
    ) -> Entity {
        let tile = self.build_component(index, storage, tilemap);

        let mut tile_entity = commands.spawn_empty();
        tile_entity.insert(tile);
        let tile_entity = tile_entity.id();
        commands.entity(tilemap).add_child(tile_entity);
        tile_entity
    }

    pub(crate) fn build_component(
        &self,
        index: IVec2,
        storage: &TilemapStorage,
        tilemap: Entity,
    ) -> Tile {
        let indices = storage.transform_index(index);
        Tile {
            tilemap_id: tilemap,
            chunk_index: indices.0,
            in_chunk_index: indices.1,
            index,
            texture: self.texture.clone(),
            color: self.color,
        }
    }
}

#[derive(Debug, Clone, Reflect)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub enum TileTexture {
    Static(Vec<TileLayer>),
    Animated(u32),
}

#[derive(Component, Clone, Debug, Reflect)]
pub struct Tile {
    pub tilemap_id: Entity,
    pub chunk_index: IVec2,
    pub in_chunk_index: UVec2,
    pub index: IVec2,
    pub texture: TileTexture,
    pub color: Vec4,
}

#[derive(Debug, Clone, Reflect)]
pub struct TileBuffer {
    pub(crate) tiles: HashMap<IVec2, TileBuilder>,
    pub(crate) aabb: IAabb2d,
}

impl TileBuffer {
    pub fn new() -> Self {
        Self {
            tiles: HashMap::new(),
            aabb: IAabb2d::default(),
        }
    }

    pub fn set(&mut self, index: IVec2, tile: TileBuilder) {
        self.tiles.insert(index, tile);
        self.aabb.expand_to_contain(index);
    }

    /// Warning: this method will cause aabb to be recalculated.
    pub fn remove(&mut self, index: IVec2) {
        self.tiles.remove(&index);
        self.recalculate_aabb();
    }

    pub fn get(&self, index: IVec2) -> Option<&TileBuilder> {
        self.tiles.get(&index)
    }

    pub fn get_mut(&mut self, index: IVec2) -> Option<&mut TileBuilder> {
        self.tiles.get_mut(&index)
    }

    pub fn recalculate_aabb(&mut self) {
        self.aabb = IAabb2d::default();
        for (index, _) in self.tiles.iter() {
            self.aabb.expand_to_contain(*index);
        }
    }
    
    #[inline]
    pub fn aabb(&self) -> IAabb2d {
        self.aabb
    }
}

pub fn tile_updater(
    commands: ParallelCommands,
    mut tiles_query: Query<(Entity, &mut Tile, &TileUpdater)>,
) {
    tiles_query
        .par_iter_mut()
        .for_each(|(entity, mut tile, updater)| {
            if let Some(layer) = &updater.layer {
                if let TileTexture::Static(ref mut tex) = tile.texture {
                    match layer.position {
                        TileLayerPosition::Top => {
                            tex.push(layer.layer);
                        }
                        TileLayerPosition::Bottom => {
                            tex.insert(0, layer.layer);
                        }
                        TileLayerPosition::Index(i) => {
                            if i >= tex.len() {
                                tex.resize(i + 1, TileLayer::new());
                            }
                            tex[i] = layer.layer;
                        }
                    }
                }
            }
            if let Some(color) = updater.color {
                tile.color = color;
            }
            commands.command_scope(|mut c| {
                c.entity(entity).remove::<TileUpdater>();
            });
        });
}
