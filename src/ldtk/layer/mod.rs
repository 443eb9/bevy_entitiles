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
        layer::TileLayer,
        map::TilemapBuilder,
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

pub struct LdtkLayers<'a> {
    pub identifier: String,
    pub ty: LdtkLoaderMode,
    pub level_entity: Entity,
    pub layers: Vec<Option<(TilemapPattern, TilemapTexture, LayerIid, f32)>>,
    pub tilesets: &'a HashMap<i32, TilemapTexture>,
    pub translation: Vec2,
    pub base_z_index: i32,
    pub background: SpriteBundle,
    #[cfg(feature = "algorithm")]
    pub path_layer: Option<(
        path::LdtkPathLayer,
        crate::tilemap::algorithm::path::PathTilemap,
    )>,
    #[cfg(any(feature = "physics_xpbd", feature = "physics_rapier"))]
    pub physics_layer: Option<(physics::LdtkPhysicsLayer, physics::LdtkPhysicsAabbs)>,
}

impl<'a> LdtkLayers<'a> {
    pub fn new(
        identifier: String,
        level_entity: Entity,
        total_layers: usize,
        ldtk_assets: &'a LdtkAssets,
        translation: Vec2,
        base_z_index: i32,
        ty: LdtkLoaderMode,
        background: SpriteBundle,
    ) -> Self {
        Self {
            identifier,
            level_entity,
            layers: vec![None; total_layers],
            tilesets: &ldtk_assets.tilesets,
            translation,
            base_z_index,
            background,
            ty,
            #[cfg(feature = "algorithm")]
            path_layer: None,
            #[cfg(any(feature = "physics_xpbd", feature = "physics_rapier"))]
            physics_layer: None,
        }
    }

    pub fn set(&mut self, layer_index: usize, layer: &LayerInstance, tile: &TileInstance) {
        self.try_create_new_layer(layer_index, layer);

        let (pattern, texture, _, _) = self.layers[layer_index].as_mut().unwrap();
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

    fn try_create_new_layer(&mut self, layer_index: usize, layer: &LayerInstance) {
        let tileset = self
            .tilesets
            .get(&layer.tileset_def_uid.unwrap())
            .cloned()
            .unwrap();

        if self.layers[layer_index].is_some() {
            return;
        }

        self.layers[layer_index] = Some((
            TilemapPattern::new(
                Some(layer.identifier.clone()),
                UVec2 {
                    x: layer.c_wid as u32,
                    y: layer.c_hei as u32,
                },
            ),
            tileset,
            LayerIid(layer.iid.clone()),
            layer.opacity,
        ));
    }

    pub fn apply_all(&mut self, commands: &mut Commands, ldtk_patterns: &mut LdtkPatterns) {
        match self.ty {
            LdtkLoaderMode::Tilemap => {
                self.layers
                    .drain(..)
                    .filter_map(|e| if let Some(e) = e { Some(e) } else { None })
                    .enumerate()
                    .for_each(|(index, (pattern, texture, iid, opacity))| {
                        let mut tilemap = TilemapBuilder::new(
                            TileType::Square,
                            texture.desc.tile_size.as_vec2(),
                            pattern.label.clone().unwrap(),
                        )
                        .with_texture(texture)
                        .with_translation(self.translation)
                        .with_z_index(self.base_z_index - index as i32 - 1)
                        .with_layer_opacities([opacity; 4])
                        .build(commands);

                        pattern.apply(
                            commands,
                            IVec2 {
                                x: 0,
                                y: -(pattern.size.y as i32),
                            },
                            &mut tilemap,
                        );

                        #[cfg(feature = "algorithm")]
                        if let Some((path_layer, path_tilemap)) = &self.path_layer {
                            if path_layer.parent == tilemap.name {
                                commands.entity(tilemap.id).insert(path_tilemap.clone());
                            }
                        }

                        #[cfg(any(feature = "physics_xpbd", feature = "physics_rapier"))]
                        if let Some((physics_layer, aabbs)) = &self.physics_layer {
                            if physics_layer.parent == tilemap.name {
                                aabbs.generate_colliders(
                                    commands,
                                    &tilemap,
                                    physics_layer.frictions.as_ref(),
                                );
                            }
                        }

                        commands.entity(self.level_entity).add_child(tilemap.id);
                        commands.entity(tilemap.id).insert((tilemap, iid));
                    });

                let bg = commands.spawn(self.background.clone()).id();
                commands.entity(self.level_entity).add_child(bg);
            }
            LdtkLoaderMode::MapPattern => {
                let level = self
                    .layers
                    .drain(..)
                    .filter_map(|p| {
                        #[allow(unused_mut)]
                        if let Some(mut p) = p {
                            #[cfg(feature = "algorithm")]
                            if let Some((path_layer, path_tilemap)) = &self.path_layer {
                                if path_layer.parent == p.0.label.clone().unwrap() {
                                    p.0.path_tiles = Some(
                                        path_tilemap
                                            .storage
                                            .clone()
                                            .into_iter()
                                            .map(|(k, v)| (k, v.into()))
                                            .collect(),
                                    );
                                }
                            }

                            Some((p.0, p.1))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();

                #[cfg(any(feature = "physics_xpbd", feature = "physics_rapier"))]
                {
                    let (physics_layer, aabbs) = self.physics_layer.as_ref().unwrap();
                    ldtk_patterns.insert_physics_aabbs(self.identifier.clone(), aabbs.clone());
                    ldtk_patterns.frictions = physics_layer.frictions.clone();
                }

                ldtk_patterns.insert(self.identifier.clone(), level, self.background.clone());
                commands.entity(self.level_entity).despawn_recursive();
            }
        }
    }

    #[cfg(feature = "algorithm")]
    pub fn assign_path_layer(
        &mut self,
        path: path::LdtkPathLayer,
        tilemap: crate::tilemap::algorithm::path::PathTilemap,
    ) {
        self.path_layer = Some((path, tilemap));
    }

    #[cfg(any(feature = "physics_xpbd", feature = "physics_rapier"))]
    pub fn assign_physics_layer(
        &mut self,
        physics: physics::LdtkPhysicsLayer,
        aabbs: physics::LdtkPhysicsAabbs,
    ) {
        self.physics_layer = Some((physics, aabbs));
    }
}
