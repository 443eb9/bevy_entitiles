use bevy::{
    ecs::{entity::Entity, system::Commands},
    hierarchy::{BuildChildren, DespawnRecursiveExt},
    math::{IVec2, UVec2, Vec2, Vec4},
    sprite::SpriteBundle,
    utils::HashMap,
};

use crate::{
    render::texture::TilemapTexture,
    serializing::pattern::TilemapPattern,
    tilemap::{
        layer::{LayerUpdater, TileLayer, TileLayerPosition, TileUpdater},
        map::{Tilemap, TilemapBuilder},
        tile::{TileBuilder, TileTexture, TileType},
    },
};

use super::{
    components::LayerIid,
    json::level::{LayerInstance, TileInstance},
    resources::{LdtkAssets, LdtkPatterns},
    LdtkLoaderMode,
};

#[cfg(feature = "algorithm")]
pub mod path;
#[cfg(any(feature = "physics_xpbd", feature = "physics_rapier"))]
pub mod physics;

#[derive(Clone)]
pub enum LdtkLayer {
    Tilemap(Vec<Option<Tilemap>>),
    MapPattern(Vec<Option<(TilemapPattern, TilemapTexture)>>),
}

pub struct LdtkLayers<'a> {
    pub identifier: String,
    pub level_entity: Entity,
    pub layers: LdtkLayer,
    pub tilesets: &'a HashMap<i32, TilemapTexture>,
    pub translation: Vec2,
    pub base_z_index: i32,
    pub background: SpriteBundle,
}

impl<'a> LdtkLayers<'a> {
    pub fn new(
        identifier: String,
        level_entity: Entity,
        total_layers: usize,
        ldtk_assets: &'a LdtkAssets,
        translation: Vec2,
        base_z_index: i32,
        mode: LdtkLoaderMode,
        background: SpriteBundle,
    ) -> Self {
        Self {
            identifier,
            level_entity,
            layers: match mode {
                LdtkLoaderMode::Tilemap => LdtkLayer::Tilemap(vec![None; total_layers]),
                LdtkLoaderMode::MapPattern => LdtkLayer::MapPattern(vec![None; total_layers]),
            },
            tilesets: &ldtk_assets.tilesets,
            translation,
            base_z_index,
            background,
        }
    }

    pub fn set(
        &mut self,
        commands: &mut Commands,
        layer_index: usize,
        layer: &LayerInstance,
        tile: &TileInstance,
    ) {
        self.try_create_new_layer(commands, layer_index, layer);

        match &mut self.layers {
            LdtkLayer::Tilemap(tilemaps) => {
                let tilemap = tilemaps[layer_index].as_mut().unwrap();

                let tile_size = tilemap.texture.as_ref().unwrap().desc.tile_size;
                let tile_index = IVec2 {
                    x: tile.px[0] / tile_size.x as i32,
                    y: tilemap.size.y as i32 - 1 - tile.px[1] / tile_size.y as i32,
                };
                if tilemap.is_out_of_tilemap_ivec(tile_index) {
                    return;
                }

                let tile_index = tile_index.as_uvec2();

                if tilemap.get(tile_index).is_none() {
                    let builder = TileBuilder::new()
                        .with_layer(
                            0,
                            TileLayer::new()
                                .with_texture_index(tile.tile_id as u32)
                                .with_flip_raw(tile.flip as u32),
                        )
                        .with_color(Vec4::new(1., 1., 1., tile.alpha));

                    tilemap.set(commands, tile_index, builder);
                    (0..4).into_iter().for_each(|i| {
                        tilemap.set_layer_opacity(i, layer.opacity);
                    });
                } else {
                    tilemap.update(
                        commands,
                        tile_index,
                        TileUpdater {
                            layer: Some(LayerUpdater {
                                position: TileLayerPosition::Top,
                                layer: TileLayer::new().with_texture_index(tile.tile_id as u32),
                            }),
                            ..Default::default()
                        },
                    );
                }
            }
            LdtkLayer::MapPattern(patterns) => {
                let (pattern, texture) = patterns[layer_index].as_mut().unwrap();
                let tile_size = texture.desc.tile_size;
                let tile_index = IVec2 {
                    x: tile.px[0] / tile_size.x as i32,
                    y: pattern.size.y as i32 - 1 - tile.px[1] / tile_size.y as i32,
                };
                if pattern.is_index_oobi(tile_index) {
                    return;
                }

                if let Some(ser_tile) = pattern.get_mut(tile_index.as_uvec2()) {
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
                    pattern.set(tile_index.as_uvec2(), Some(builder.into()));
                }
            }
        }
    }

    fn try_create_new_layer(
        &mut self,
        commands: &mut Commands,
        layer_index: usize,
        layer: &LayerInstance,
    ) {
        let tileset = self
            .tilesets
            .get(&layer.tileset_def_uid.unwrap())
            .cloned()
            .unwrap();

        match &mut self.layers {
            LdtkLayer::Tilemap(tilemaps) => {
                if tilemaps[layer_index].is_some() {
                    return;
                }

                let tilemap = TilemapBuilder::new(
                    TileType::Square,
                    UVec2 {
                        x: layer.c_wid as u32,
                        y: layer.c_hei as u32,
                    },
                    tileset.desc.tile_size.as_vec2(),
                    layer.identifier.clone(),
                )
                .with_texture(tileset)
                .with_translation(self.translation)
                .with_z_index(self.base_z_index - layer_index as i32)
                .build(commands);

                commands
                    .entity(tilemap.id)
                    .insert(LayerIid(layer.iid.clone()));
                commands.entity(self.level_entity).add_child(tilemap.id());

                tilemaps[layer_index] = Some(tilemap);
            }
            LdtkLayer::MapPattern(patterns) => {
                if patterns[layer_index].is_some() {
                    return;
                }

                patterns[layer_index] = Some((
                    TilemapPattern::new(
                        Some(layer.identifier.clone()),
                        UVec2 {
                            x: layer.c_wid as u32,
                            y: layer.c_hei as u32,
                        },
                    ),
                    tileset,
                ));
            }
        }
    }

    pub fn apply_all(&mut self, commands: &mut Commands, ldtk_patterns: &mut LdtkPatterns) {
        match &mut self.layers {
            LdtkLayer::Tilemap(tilemaps) => {
                for layer in tilemaps.drain(..) {
                    if let Some(tm) = layer {
                        commands.entity(tm.id).insert(tm);
                    }
                }

                let bg = commands.spawn(self.background.clone()).id();
                commands.entity(self.level_entity).add_child(bg);
            }
            LdtkLayer::MapPattern(patterns) => {
                let level = patterns
                    .drain(..)
                    .filter_map(|p| if let Some(p) = p { Some(p) } else { None })
                    .collect::<Vec<_>>();

                ldtk_patterns.insert(self.identifier.clone(), level, self.background.clone());
                commands.entity(self.level_entity).despawn_recursive();
            }
        }
    }

    #[cfg(feature = "algorithm")]
    pub fn apply_path_layer(
        &mut self,
        commands: &mut Commands,
        path: &path::LdtkPathLayer,
        tilemap: crate::tilemap::algorithm::path::PathTilemap,
    ) {
        let path_map = self.find_layer(&path.parent, &path.identifier);
        commands.entity(path_map.id).insert(tilemap);
    }

    #[cfg(any(feature = "physics_xpbd", feature = "physics_rapier"))]
    pub fn apply_physics_layer(
        &mut self,
        commands: &mut Commands,
        physics: &physics::LdtkPhysicsLayer,
        aabbs: Vec<(i32, (UVec2, UVec2))>,
    ) {
        let physics_map = self.find_layer(&physics.parent, &physics.identifier);
        aabbs
            .into_iter()
            .map(|(i, (min, max))| {
                let left_top = physics_map.index_inf_to_world(IVec2 {
                    x: min.x as i32,
                    y: (physics_map.size.y - 1 - min.y + 1) as i32,
                }) - physics_map.transform.translation;
                let right_btm = physics_map.index_inf_to_world(IVec2 {
                    x: (max.x + 1) as i32,
                    y: (physics_map.size.y - 1 - max.y) as i32,
                }) - physics_map.transform.translation;
                (
                    i,
                    crate::math::aabb::Aabb2d {
                        min: Vec2 {
                            x: left_top.x,
                            y: right_btm.y,
                        },
                        max: Vec2 {
                            x: right_btm.x,
                            y: left_top.y,
                        },
                    },
                )
            })
            .for_each(|(i, aabb)| {
                let mut collider = commands.spawn(bevy::transform::TransformBundle {
                    local: bevy::transform::components::Transform::from_translation(
                        aabb.center().extend(0.),
                    ),
                    ..Default::default()
                });
                collider.set_parent(physics_map.id);

                #[cfg(feature = "physics_xpbd")]
                {
                    collider.insert((
                        bevy_xpbd_2d::components::Collider::cuboid(aabb.width(), aabb.height()),
                        bevy_xpbd_2d::components::RigidBody::Static,
                    ));
                    if let Some(coe) = physics.frictions.as_ref().and_then(|f| f.get(&i)) {
                        collider.insert(bevy_xpbd_2d::components::Friction {
                            dynamic_coefficient: *coe,
                            static_coefficient: *coe,
                            ..Default::default()
                        });
                    }
                }

                #[cfg(feature = "physics_rapier")]
                {
                    collider.insert(bevy_rapier2d::geometry::Collider::cuboid(
                        aabb.width() / 2.,
                        aabb.height() / 2.,
                    ));
                    if let Some(coe) = physics.frictions.as_ref().and_then(|f| f.get(&i)) {
                        collider.insert(bevy_rapier2d::geometry::Friction {
                            coefficient: *coe,
                            ..Default::default()
                        });
                    }
                }
            });
    }

    fn find_layer(&mut self, parent: &String, layer: &String) -> &Tilemap {
        match &mut self.layers {
            LdtkLayer::Tilemap(tilemaps) => tilemaps
                .iter()
                .find(|map| {
                    if let Some(map) = map {
                        map.name == *parent
                    } else {
                        false
                    }
                })
                .unwrap_or_else(|| panic!("Missing parent {} for layer {}!", parent, layer))
                .as_ref()
                .unwrap_or_else(|| {
                    panic!(
                        "The parent layer {} for layer {} is not a rendered layer!",
                        parent, layer
                    )
                }),
            LdtkLayer::MapPattern(..) => todo!(),
        }
    }
}
