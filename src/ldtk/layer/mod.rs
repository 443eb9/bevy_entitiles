use bevy::{
    asset::AssetServer,
    ecs::{entity::Entity, system::Commands},
    hierarchy::DespawnRecursiveExt,
    math::{IVec2, Vec2, Vec4},
    prelude::SpatialBundle,
    sprite::SpriteBundle,
    transform::components::Transform,
    utils::HashMap,
};

use crate::{
    math::aabb::IAabb2d,
    serializing::pattern::TilemapPattern,
    tilemap::{
        buffers::TileBuffer,
        bundles::TilemapBundle,
        map::{
            TileRenderSize, TilemapLayerOpacities, TilemapName, TilemapSlotSize, TilemapStorage,
            TilemapTexture, TilemapTransform, TilemapType,
        },
        tile::{TileBuilder, TileLayer, TileTexture},
    },
    DEFAULT_CHUNK_SIZE,
};

use super::{
    components::{LayerIid, LdtkLoadedLevel, LevelIid},
    entities::{LdtkEntityRegistry, PackedLdtkEntity},
    json::level::{LayerInstance, Level, TileInstance},
    resources::{LdtkAssets, LdtkLevelManager, LdtkPatterns},
    LdtkLoaderMode,
};

#[cfg(feature = "algorithm")]
pub mod path;
#[cfg(feature = "physics")]
pub mod physics;

pub type LayerOpacity = f32;

pub struct LdtkLayers<'a> {
    pub ty: LdtkLoaderMode,
    pub level_entity: Entity,
    pub layers: Vec<Option<(TilemapPattern, TilemapTexture, LayerIid, LayerOpacity)>>,
    pub entities: Vec<PackedLdtkEntity>,
    pub tilesets: &'a HashMap<i32, TilemapTexture>,
    pub translation: Vec2,
    pub base_z_index: i32,
    pub background: SpriteBundle,
    #[cfg(feature = "algorithm")]
    pub path_layer: Option<(
        path::LdtkPathLayer,
        HashMap<IVec2, crate::tilemap::algorithm::path::PathTile>,
    )>,
    #[cfg(feature = "physics")]
    pub physics_layer: Option<(physics::LdtkPhysicsLayer, physics::LdtkPhysicsAabbs)>,
}

impl<'a> LdtkLayers<'a> {
    pub fn new(
        level_entity: Entity,
        total_layers: usize,
        ldtk_assets: &'a LdtkAssets,
        translation: Vec2,
        base_z_index: i32,
        ty: LdtkLoaderMode,
        background: SpriteBundle,
    ) -> Self {
        Self {
            level_entity,
            layers: vec![None; total_layers],
            entities: vec![],
            tilesets: &ldtk_assets.tilesets,
            translation,
            base_z_index,
            background,
            ty,
            #[cfg(feature = "algorithm")]
            path_layer: None,
            #[cfg(feature = "physics")]
            physics_layer: None,
        }
    }

    pub fn set_tile(&mut self, layer_index: usize, layer: &LayerInstance, tile: &TileInstance) {
        self.try_create_new_layer(layer_index, layer);

        let (pattern, texture, _, _) = self.layers[layer_index].as_mut().unwrap();
        let tile_size = texture.desc.tile_size;
        let tile_index = IVec2 {
            x: tile.px[0] / tile_size.x as i32,
            y: -tile.px[1] / tile_size.y as i32,
        };

        if let Some(ser_tile) = pattern.tiles.get_mut(tile_index) {
            let TileTexture::Static(tile_layers) = &mut ser_tile.texture else {
                unreachable!()
            };
            tile_layers.push(TileLayer::new().with_texture_index(tile.tile_id as u32));
        } else {
            let builder = TileBuilder::new()
                .with_layer(
                    0,
                    TileLayer::new()
                        .with_texture_index(tile.tile_id as u32)
                        .with_flip_raw(tile.flip as u32),
                )
                .with_color(Vec4::new(1., 1., 1., tile.alpha));
            pattern.tiles.tiles.insert(tile_index, builder);
        }
    }

    pub fn set_entity(&mut self, entity: PackedLdtkEntity) {
        self.entities.push(entity);
    }

    fn try_create_new_layer(&mut self, layer_index: usize, layer: &LayerInstance) {
        let tileset = self
            .tilesets
            .get(&layer.tileset_def_uid.unwrap())
            .cloned()
            .unwrap();

        if self.layers[layer_index].is_some() {
            return;
        }

        let aabb = IAabb2d {
            min: IVec2::new(0, -layer.c_hei + 1),
            max: IVec2::new(layer.c_wid - 1, 0),
        };

        self.layers[layer_index] = Some((
            TilemapPattern {
                label: Some(layer.identifier.clone()),
                tiles: TileBuffer {
                    aabb,
                    tiles: HashMap::new(),
                },
                #[cfg(feature = "algorithm")]
                path_tiles: TileBuffer {
                    aabb,
                    tiles: HashMap::new(),
                },
            },
            tileset,
            LayerIid(layer.iid.clone()),
            layer.opacity,
        ));
    }

    pub fn apply_all(
        &mut self,
        commands: &mut Commands,
        ldtk_patterns: &mut LdtkPatterns,
        level: &Level,
        entity_registry: &LdtkEntityRegistry,
        manager: &LdtkLevelManager,
        ldtk_assets: &LdtkAssets,
        asset_server: &AssetServer,
    ) {
        match self.ty {
            LdtkLoaderMode::Tilemap => {
                let mut layers = HashMap::with_capacity(self.layers.len());
                let mut entities = HashMap::with_capacity(self.entities.len());
                let mut colliders = Vec::new();

                self.entities.drain(..).for_each(|entity| {
                    let mut ldtk_entity = commands.spawn(entity.transform.clone());
                    entities.insert(entity.iid.clone(), ldtk_entity.id());
                    entity.instantiate(
                        &mut ldtk_entity,
                        entity_registry,
                        manager,
                        ldtk_assets,
                        asset_server,
                    );
                });

                self.layers
                .drain(..)
                .enumerate()
                .filter_map(|(i,e)| {
                    if let Some(e) = e {
                        Some((i,e))
                    }else {
                        None
                    }
                })
                    .for_each(|(index, (pattern, texture, iid, opacity))| {
                        let tilemap_entity = commands.spawn_empty().id();
                        let mut tilemap = TilemapBundle {
                            name: TilemapName(pattern.label.clone().unwrap()),
                            ty: TilemapType::Square,
                            tile_render_size: TileRenderSize(texture.desc.tile_size.as_vec2()),
                            slot_size: TilemapSlotSize(texture.desc.tile_size.as_vec2()),
                            texture: texture.clone(),
                            storage: TilemapStorage::new(DEFAULT_CHUNK_SIZE, tilemap_entity),
                            tilemap_transform: TilemapTransform {
                                translation: self.translation,
                                z_index: self.base_z_index - index as i32 - 1,
                                ..Default::default()
                            },
                            layer_opacities: TilemapLayerOpacities([opacity; 4].into()),
                            ..Default::default()
                        };

                        tilemap
                            .storage
                            .fill_with_buffer(commands, IVec2::NEG_Y, pattern.tiles);

                        #[cfg(feature = "algorithm")]
                        if let Some((path_layer, path_tilemap)) = &self.path_layer {
                            if path_layer.parent == tilemap.name.0 {
                                commands.entity(tilemap_entity).insert(
                                    crate::tilemap::algorithm::path::PathTilemap {
                                        storage:
                                            crate::tilemap::chunking::storage::ChunkedStorage::from_mapper(
                                                path_tilemap.clone(),
                                                None,
                                            ),
                                    },
                                );
                            }
                        }

                        #[cfg(feature = "physics")]
                        if let Some((physics_layer, aabbs)) = &self.physics_layer {
                            if physics_layer.parent == tilemap.name.0 {
                                colliders = aabbs.generate_colliders(
                                    commands,
                                    tilemap_entity,
                                    &tilemap.ty,
                                    &tilemap.tilemap_transform,
                                    &tilemap.tile_pivot,
                                    &tilemap.slot_size,
                                    physics_layer.frictions.as_ref(),
                                    Vec2::ZERO,
                                );
                            }
                        }

                        commands.entity(tilemap_entity).insert((tilemap, iid.clone()));
                        layers.insert(iid, tilemap_entity);
                    });

                let bg = commands.spawn(self.background.clone()).id();

                commands.entity(self.level_entity).insert((
                    LdtkLoadedLevel {
                        identifier: level.identifier.clone(),
                        layers,
                        entities,
                        background: bg,
                        colliders,
                    },
                    SpatialBundle {
                        transform: Transform::from_translation(self.translation.extend(0.)),
                        ..Default::default()
                    },
                    LevelIid(level.iid.clone()),
                ));
            }
            LdtkLoaderMode::MapPattern => {
                let level_pack = self
                    .layers
                    .drain(..)
                    .filter_map(|p| {
                        #[allow(unused_mut)]
                        if let Some(mut p) = p {
                            #[cfg(feature = "algorithm")]
                            if let Some((path_layer, path_tiles)) = &self.path_layer {
                                if path_layer.parent == p.0.label.clone().unwrap() {
                                    p.0.path_tiles.tiles = path_tiles.clone();
                                }
                            }

                            Some((p.0, p.1))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();

                #[cfg(feature = "physics")]
                {
                    let (physics_layer, aabbs) = self.physics_layer.as_ref().unwrap();
                    ldtk_patterns.insert_physics_aabbs(level.identifier.clone(), aabbs.clone());
                    ldtk_patterns.frictions = physics_layer.frictions.clone();
                }

                ldtk_patterns.insert(
                    level.identifier.clone(),
                    level_pack,
                    self.background.clone(),
                );
                commands.entity(self.level_entity).despawn_recursive();
            }
        }
    }

    #[cfg(feature = "algorithm")]
    pub fn assign_path_layer(
        &mut self,
        path: path::LdtkPathLayer,
        tilemap: HashMap<IVec2, crate::tilemap::algorithm::path::PathTile>,
    ) {
        self.path_layer = Some((path, tilemap));
    }

    #[cfg(feature = "physics")]
    pub fn assign_physics_layer(
        &mut self,
        physics: physics::LdtkPhysicsLayer,
        aabbs: physics::LdtkPhysicsAabbs,
    ) {
        self.physics_layer = Some((physics, aabbs));
    }
}
