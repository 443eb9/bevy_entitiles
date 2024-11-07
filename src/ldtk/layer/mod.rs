use bevy::{
    asset::{AssetId, AssetServer, Assets},
    color::LinearRgba,
    ecs::{
        entity::Entity,
        system::{Commands, EntityCommands},
    },
    math::{IVec2, Vec2},
    prelude::{Component, SpatialBundle},
    sprite::SpriteBundle,
    transform::components::Transform,
    utils::HashMap,
};

use crate::{
    ldtk::{
        components::{EntityIid, LayerIid, LdtkLoadedLevel, LdtkTempTransform, LevelIid},
        events::LdtkLevelLoaderMode,
        json::{
            field::FieldInstance,
            level::{EntityInstance, LayerInstance, Level, TileInstance},
        },
        resources::{LdtkAssets, LdtkLevelConfig, LdtkPatterns},
        traits::{LdtkEntityRegistry, LdtkEntityTagRegistry},
    },
    math::GridRect,
    render::material::StandardTilemapMaterial,
    serializing::pattern::TilemapPattern,
    tilemap::{
        buffers::TileBuffer,
        bundles::StandardTilemapBundle,
        map::{
            TileRenderSize, TilemapLayerOpacities, TilemapName, TilemapSlotSize, TilemapStorage,
            TilemapTexture, TilemapTextures, TilemapTransform, TilemapType,
        },
        tile::{TileBuilder, TileFlip, TileLayer, TileTexture},
    },
    DEFAULT_CHUNK_SIZE,
};

#[cfg(feature = "algorithm")]
use crate::{
    algorithm::pathfinding::PathTilemaps,
    tilemap::{algorithm::path::PathTilemap, chunking::storage::ChunkedStorage},
};

#[cfg(feature = "physics")]
use crate::tilemap::physics::{DataPhysicsTilemap, SerializablePhysicsSource};
#[cfg(feature = "physics")]
use bevy::math::UVec2;

#[cfg(feature = "algorithm")]
pub mod path;
#[cfg(feature = "physics")]
pub mod physics;

#[derive(Debug, Clone)]
pub struct PackedLdtkEntity {
    pub instance: EntityInstance,
    pub fields: HashMap<String, FieldInstance>,
    pub iid: EntityIid,
    pub transform: LdtkTempTransform,
}

impl PackedLdtkEntity {
    pub fn instantiate(
        self,
        commands: &mut EntityCommands,
        entity_registry: &LdtkEntityRegistry,
        entity_tag_registry: &LdtkEntityTagRegistry,
        config: &LdtkLevelConfig,
        ldtk_assets: &LdtkAssets,
        asset_server: &AssetServer,
    ) {
        let phantom_entity = {
            if let Some(e) = entity_registry.get(&self.instance.identifier) {
                e
            } else if !config.ignore_unregistered_entities {
                panic!(
                    "Could not find entity type with entity identifier: {}! \
                    You need to register it using App::register_ldtk_entity::<T>() first!",
                    self.instance.identifier
                );
            } else {
                return;
            }
        };

        self.instance.tags.iter().for_each(|tag| {
            if let Some(entity_tag) = entity_tag_registry.get(tag) {
                entity_tag.add_tag(commands);
            } else if !config.ignore_unregistered_entity_tags {
                panic!(
                    "Could not find entity tag with tag: {}! \
                    You need to register it using App::register_ldtk_entity_tag::<T>() first! \
                    Or call LdtkLevelManager::ignore_unregistered_entity_tags to ignore.",
                    tag
                );
            }
        });

        phantom_entity.spawn(
            commands,
            &self.instance,
            &self.fields,
            asset_server,
            ldtk_assets,
        )
    }
}

pub type LayerOpacity = f32;

#[derive(Component)]
pub struct LdtkLayers {
    pub assets_id: AssetId<LdtkAssets>,
    pub ty: LdtkLevelLoaderMode,
    pub level_entity: Entity,
    pub level: Level,
    pub layers: Vec<Option<(TilemapPattern, TilemapTexture, LayerIid, LayerOpacity)>>,
    pub entities: Vec<PackedLdtkEntity>,
    pub tilesets: HashMap<i32, TilemapTexture>,
    pub translation: Vec2,
    pub base_z_index: f32,
    pub background: SpriteBundle,
    #[cfg(feature = "algorithm")]
    pub path_layer: Option<(
        path::LdtkPathLayer,
        HashMap<IVec2, crate::tilemap::algorithm::path::PathTile>,
    )>,
    #[cfg(feature = "physics")]
    pub physics_layer: Option<(physics::LdtkPhysicsLayer, Vec<i32>, UVec2)>,
}

impl LdtkLayers {
    pub fn new(
        level_entity: Entity,
        level: &Level,
        total_layers: usize,
        assets_id: AssetId<LdtkAssets>,
        ldtk_assets: &LdtkAssets,
        translation: Vec2,
        base_z_index: f32,
        ty: LdtkLevelLoaderMode,
        background: SpriteBundle,
    ) -> Self {
        Self {
            assets_id,
            level_entity,
            level: level.clone(),
            layers: vec![None; total_layers],
            entities: vec![],
            tilesets: ldtk_assets.tilesets.clone(),
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

    pub fn set_tile(
        &mut self,
        layer_index: usize,
        layer: &LayerInstance,
        tile: &TileInstance,
        config: &LdtkLevelConfig,
        patterns: &LdtkPatterns,
        mode: &LdtkLevelLoaderMode,
    ) {
        self.try_create_new_layer(layer_index, layer);

        let (pattern, texture, _, _) = self.layers[layer_index].as_mut().unwrap();
        let tile_size = texture.desc.tile_size;
        let tile_index = IVec2 {
            x: tile.px[0] / tile_size.x as i32,
            y: match mode {
                LdtkLevelLoaderMode::Tilemap => -tile.px[1] / tile_size.y as i32 - 1,
                LdtkLevelLoaderMode::MapPattern => {
                    patterns.pattern_size.y as i32 - tile.px[1] / tile_size.y as i32 - 1
                }
            },
        };
        let atlas_index = tile.tile_id;

        if let Some(ser_tile) = pattern.tiles.get_mut(tile_index) {
            let TileTexture::Static(tile_layers) = &mut ser_tile.texture else {
                panic!(
                    "Trying to insert multiple layers into a animated tile at {}!",
                    tile_index
                );
            };
            tile_layers.push(TileLayer {
                atlas_index,
                ..Default::default()
            });
        } else {
            let mut builder = TileBuilder::new().with_tint(LinearRgba::new(1., 1., 1., tile.alpha));
            builder = {
                if let Some(anim) = config.animation_mapper.get(&(atlas_index as u32)) {
                    let animation = pattern.animations.register(anim.clone());
                    builder.with_animation(animation)
                } else {
                    let flip = tile.flip.reverse_bits() >> 30 & 0b11;
                    builder.with_layer(
                        0,
                        TileLayer {
                            #[cfg(feature = "atlas")]
                            texture_index: 0,
                            atlas_index,
                            flip: TileFlip::from_bits(flip as u32).unwrap(),
                        },
                    )
                }
            };

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

        let aabb = GridRect::from_min_max(
            IVec2::new(0, -layer.c_hei + 1),
            IVec2::new(layer.c_wid - 1, 0),
        );

        self.layers[layer_index] = Some((
            TilemapPattern {
                label: Some(layer.identifier.clone()),
                tiles: TileBuffer {
                    aabb,
                    tiles: HashMap::new(),
                },
                animations: Default::default(),
                #[cfg(feature = "algorithm")]
                path_tiles: TileBuffer {
                    aabb,
                    tiles: HashMap::new(),
                },
                #[cfg(feature = "physics")]
                physics_tiles: SerializablePhysicsSource::Buffer(TileBuffer {
                    aabb,
                    tiles: HashMap::new(),
                }),
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
        entity_registry: &LdtkEntityRegistry,
        entity_tag_registry: &LdtkEntityTagRegistry,
        config: &LdtkLevelConfig,
        ldtk_assets: &LdtkAssets,
        asset_server: &AssetServer,
        material_assets: &mut Assets<StandardTilemapMaterial>,
        textures_assets: &mut Assets<TilemapTextures>,
        #[cfg(feature = "algorithm")] path_tilemaps: &mut PathTilemaps,
    ) {
        match self.ty {
            LdtkLevelLoaderMode::Tilemap => {
                let mut layers = HashMap::with_capacity(self.layers.len());
                let mut entities = HashMap::with_capacity(self.entities.len());

                self.entities.drain(..).for_each(|entity| {
                    let mut ldtk_entity =
                        commands.spawn((entity.transform.clone(), entity.iid.clone()));
                    entities.insert(entity.iid.clone(), ldtk_entity.id());
                    entity.instantiate(
                        &mut ldtk_entity,
                        entity_registry,
                        entity_tag_registry,
                        config,
                        ldtk_assets,
                        asset_server,
                    );
                });

                self.layers
                    .drain(..)
                    .enumerate()
                    .filter_map(|(i, e)| if let Some(e) = e { Some((i, e)) } else { None })
                    .for_each(|(index, (pattern, texture, iid, opacity))| {
                        let tilemap_entity = commands.spawn_empty().id();
                        let mut tilemap = StandardTilemapBundle {
                            name: TilemapName(pattern.label.clone().unwrap()),
                            ty: TilemapType::Square,
                            tile_render_size: TileRenderSize(texture.desc.tile_size.as_vec2()),
                            slot_size: TilemapSlotSize(texture.desc.tile_size.as_vec2()),
                            textures: textures_assets
                                .add(TilemapTextures::single(texture.clone(), config.filter_mode)),
                            storage: TilemapStorage::new(DEFAULT_CHUNK_SIZE, tilemap_entity),
                            transform: TilemapTransform {
                                translation: self.translation,
                                z_index: self.base_z_index - index as f32 - 1.,
                                ..Default::default()
                            },
                            material: material_assets.add(StandardTilemapMaterial::default()),
                            layer_opacities: TilemapLayerOpacities([opacity; 4].into()),
                            animations: pattern.animations.clone(),
                            ..Default::default()
                        };

                        tilemap
                            .storage
                            .fill_with_buffer(commands, IVec2::ZERO, pattern.tiles);

                        #[cfg(feature = "algorithm")]
                        if let Some((path_layer, path_tilemap)) = &self.path_layer {
                            if path_layer.parent == tilemap.name.0 {
                                path_tilemaps.insert(
                                    tilemap_entity,
                                    PathTilemap {
                                        storage: ChunkedStorage::from_mapper(
                                            path_tilemap.clone(),
                                            DEFAULT_CHUNK_SIZE,
                                        ),
                                    },
                                );
                            }
                        }

                        #[cfg(feature = "physics")]
                        if let Some((physics_layer, physics_data, size)) = &self.physics_layer {
                            if pattern.label.clone().unwrap() == physics_layer.parent {
                                commands
                                    .entity(tilemap_entity)
                                    .insert(DataPhysicsTilemap::new(
                                        IVec2::new(0, -(size.y as i32)),
                                        physics_data.clone(),
                                        *size,
                                        physics_layer.air,
                                        physics_layer.tiles.clone().unwrap_or_default(),
                                    ));
                            }
                        }

                        commands
                            .entity(tilemap_entity)
                            .insert((tilemap, iid.clone()));
                        layers.insert(iid, tilemap_entity);
                    });

                let bg = commands.spawn(self.background.clone()).id();

                commands.entity(self.level_entity).insert((
                    LdtkLoadedLevel {
                        identifier: self.level.identifier.clone(),
                        layers,
                        entities,
                        background: bg,
                    },
                    SpatialBundle {
                        transform: Transform::from_translation(self.translation.extend(0.)),
                        ..Default::default()
                    },
                    LevelIid(self.level.iid.clone()),
                ));
            }
            LdtkLevelLoaderMode::MapPattern => {
                self.layers
                    .drain(..)
                    .enumerate()
                    .for_each(|(layer_index, p)| {
                        #[allow(unused_mut)]
                        let Some((mut pattern, texture, iid, _)) = p
                        else {
                            return;
                        };

                        #[cfg(feature = "algorithm")]
                        if let Some((path_layer, path_tiles)) = &self.path_layer {
                            if path_layer.parent == pattern.label.clone().unwrap() {
                                pattern.path_tiles.tiles = path_tiles.clone();
                            }
                        }

                        #[cfg(feature = "physics")]
                        if let Some((physics_layer, physics_data, size)) =
                            self.physics_layer.as_ref()
                        {
                            pattern.physics_tiles =
                                SerializablePhysicsSource::Data(DataPhysicsTilemap::new(
                                    IVec2::ZERO,
                                    physics_data.clone(),
                                    *size,
                                    physics_layer.air,
                                    physics_layer.tiles.clone().unwrap_or_default(),
                                ));
                        }

                        ldtk_patterns.add_pattern(
                            layer_index,
                            &iid,
                            pattern,
                            &Some(texture),
                            &self.level.identifier,
                        );

                        ldtk_patterns
                            .add_background(&self.level.identifier, self.background.clone());
                    });

                commands.entity(self.level_entity).despawn();
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
        physics_layer: physics::LdtkPhysicsLayer,
        physics_data: Vec<i32>,
        size: UVec2,
    ) {
        self.physics_layer = Some((physics_layer, physics_data, size));
    }
}
