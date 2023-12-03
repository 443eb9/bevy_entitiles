use bevy::{
    asset::Handle,
    ecs::{
        component::Component,
        entity::Entity,
        system::{Commands, ParallelCommands, Query},
    },
    math::{Vec2, Vec4},
    render::{render_phase::PhaseItem, render_resource::FilterMode, texture::Image},
};

use crate::render::texture::{TileUV, TilemapTextureDescriptor};

use super::tile::{InvisibleTile, TilemapTexture};

pub struct UiTileBuilder {
    pub color: Vec4,
    pub texture_index: u32,
    pub scale: Vec2,
    pub origin: Vec2,
}

impl UiTileBuilder {
    pub fn new(texture_index: u32, origin: Vec2) -> Self {
        Self {
            color: Vec4::ONE,
            texture_index,
            scale: Vec2::ONE,
            origin,
        }
    }

    pub fn with_color(&mut self, color: Vec4) -> &mut Self {
        self.color = color;
        self
    }

    pub fn with_scale(&mut self, scale: Vec2) -> &mut Self {
        self.scale = scale;
        self
    }

    pub(crate) fn build(&self, commands: &mut Commands, tilemap: &mut UiTilemap) -> Entity {
        commands
            .spawn(UiTile {
                tilemap_id: tilemap.id,
                index: tilemap.tiles.len(),
                texture_index: self.texture_index,
                scale: self.scale,
                color: self.color,
            })
            .id()
    }
}

#[derive(Component)]
pub struct UiTile {
    pub(crate) tilemap_id: Entity,
    pub(crate) index: usize,
    pub(crate) texture_index: u32,
    pub(crate) scale: Vec2,
    pub(crate) color: Vec4,
}

pub struct UiTilemapBuilder {
    pub anchor: Vec2,
    pub texture: TilemapTexture,
    pub filter_mode: FilterMode,
}

impl UiTilemapBuilder {
    pub fn new(texture: TilemapTexture, filter_mode: FilterMode) -> Self {
        Self {
            anchor: Vec2 { x: 0.5, y: 0.5 },
            texture,
            filter_mode,
        }
    }

    pub fn with_anchor(&mut self, anchor: Vec2) -> &mut Self {
        self.anchor = anchor;
        self
    }

    pub fn build(&self, commands: &mut Commands) -> (Entity, UiTilemap) {
        let mut entity = commands.spawn_empty();
        let tilemap = UiTilemap {
            id: entity.id(),
            anchor: self.anchor,
            texture: self.texture.clone(),
            tiles: vec![],
        };
        entity.insert(tilemap.clone());
        (entity.id(), tilemap)
    }
}

#[derive(Component, Clone)]
pub struct UiTilemap {
    pub(crate) id: Entity,
    pub(crate) anchor: Vec2,
    pub(crate) texture: TilemapTexture,
    pub(crate) tiles: Vec<Entity>,
}

impl UiTilemap {
    pub fn add(&mut self, commands: &mut Commands, tile: &UiTileBuilder) {
        let new_tile = tile.build(commands, self);
        self.tiles.push(new_tile);
    }

    /// This prevented from remove the same tile twice.
    /// If you are sure that you are not doing so, use `remove_unchecked` instaed.
    pub fn remove(&mut self, commands: &mut Commands, tile: &UiTile, tile_entity: Entity) {
        if self.tiles[tile.index] == tile_entity {
            self.remove_unchecked(commands, tile);
        }
    }

    pub fn remove_unchecked(&mut self, commands: &mut Commands, tile: &UiTile) {
        commands.entity(self.tiles[tile.index]).despawn();
        self.tiles.swap_remove(tile.index);
        commands
            .entity(self.tiles[tile.index])
            .insert(UiTileIndexUpdate {
                new_index: tile.index,
            });
    }

    pub fn set_visibility(&mut self, commands: &mut Commands, tile: &UiTile, visibile: bool) {
        let mut cmd = commands.entity(self.tiles[tile.index]);
        if visibile {
            cmd.remove::<InvisibleTile>();
        } else {
            cmd.insert(InvisibleTile);
        }
    }
}

#[derive(Component)]
pub(crate) struct UiTileIndexUpdate {
    pub new_index: usize,
}

pub(crate) fn update_index(
    commands: ParallelCommands,
    mut tiles_query: Query<(Entity, &mut UiTile, &UiTileIndexUpdate)>,
) {
    tiles_query
        .par_iter_mut()
        .for_each(|(entity, mut tile, new_index)| {
            tile.index = new_index.new_index;
            commands.command_scope(|mut c| {
                c.entity(entity).remove::<UiTileIndexUpdate>();
            });
        });
}
