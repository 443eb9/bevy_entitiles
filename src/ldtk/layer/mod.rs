use bevy::{
    ecs::{entity::Entity, system::Commands},
    hierarchy::BuildChildren,
    math::{IVec2, UVec2, Vec2, Vec4},
    prelude::SpatialBundle,
    transform::{components::Transform, TransformBundle},
    utils::HashMap,
};

use crate::{
    math::aabb::Aabb2d,
    render::texture::TilemapTexture,
    tilemap::{
        algorithm::path::PathTilemap,
        layer::{LayerUpdater, TileLayer, TileLayerPosition, TileUpdater},
        map::{Tilemap, TilemapBuilder},
        tile::{TileBuilder, TileType},
    },
};

use self::{path::LdtkPathLayer, physics::LdtkPhysicsLayer};

use super::{
    components::LayerIid,
    json::level::{LayerInstance, TileInstance},
    resources::LdtkAssets,
};

pub mod path;
pub mod physics;

pub struct LdtkLayers<'a> {
    pub level_entity: Entity,
    pub layers: Vec<Option<Tilemap>>,
    pub tilesets: &'a HashMap<i32, TilemapTexture>,
    pub px_size: UVec2,
    pub translation: Vec2,
    pub base_z_index: i32,
}

impl<'a> LdtkLayers<'a> {
    pub fn new(
        level_entity: Entity,
        total_layers: usize,
        px_size: UVec2,
        ldtk_assets: &'a LdtkAssets,
        translation: Vec2,
        base_z_index: i32,
    ) -> Self {
        Self {
            level_entity,
            layers: vec![None; total_layers],
            tilesets: &ldtk_assets.tilesets,
            px_size,
            translation,
            base_z_index,
        }
    }

    pub fn set(
        &mut self,
        commands: &mut Commands,
        layer_index: usize,
        layer: &LayerInstance,
        tile: &TileInstance,
    ) {
        let tilemap = match self.layers[layer_index] {
            Some(ref mut tilemap) => tilemap,
            None => self.create_new_layer(commands, layer_index, layer),
        };

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

    fn create_new_layer(
        &mut self,
        commands: &mut Commands,
        layer_index: usize,
        layer: &LayerInstance,
    ) -> &mut Tilemap {
        let tileset = self
            .tilesets
            .get(&layer.tileset_def_uid.unwrap())
            .cloned()
            .unwrap();

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
            .insert((SpatialBundle::default(), LayerIid(layer.iid.clone())));
        commands.entity(self.level_entity).add_child(tilemap.id());

        self.layers[layer_index] = Some(tilemap);
        self.layers[layer_index].as_mut().unwrap()
    }

    pub fn apply_all(&mut self, commands: &mut Commands) {
        for tilemap in self.layers.drain(..) {
            if let Some(tm) = tilemap {
                commands.entity(tm.id).insert(tm);
            }
        }
    }

    pub fn apply_path_layer(
        &mut self,
        commands: &mut Commands,
        path: &LdtkPathLayer,
        tilemap: PathTilemap,
    ) {
        let path_map = self.find_layer(&path.parent, &path.identifier);
        commands.entity(path_map.id).insert(tilemap);
    }

    pub fn apply_physics_layer(
        &mut self,
        commands: &mut Commands,
        physics: &LdtkPhysicsLayer,
        aabbs: Vec<(i32, (UVec2, UVec2))>,
    ) {
        let physics_map = self.find_layer(&physics.parent, &physics.identifier);
        aabbs
            .into_iter()
            .map(|(i, (min, max))| {
                let left_top = physics_map.index_inf_to_world(IVec2 {
                    x: min.x as i32,
                    y: (physics_map.size.y - 1 - min.y + 1) as i32,
                });
                let right_btm = physics_map.index_inf_to_world(IVec2 {
                    x: (max.x + 1) as i32,
                    y: (physics_map.size.y - 1 - max.y) as i32,
                });
                (
                    i,
                    Aabb2d {
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
                let mut collider = commands.spawn(TransformBundle {
                    local: Transform::from_translation(aabb.center().extend(0.)),
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

    fn find_layer(&self, parent: &String, layer: &String) -> &Tilemap {
        self.layers
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
            })
    }
}
