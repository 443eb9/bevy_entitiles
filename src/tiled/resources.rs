use std::{
    f32::consts::PI,
    path::{Path, PathBuf},
};

use bevy::{
    asset::{AssetServer, Assets, Handle},
    ecs::{
        entity::Entity,
        system::{Commands, Resource},
    },
    log::{error, warn},
    math::{UVec2, Vec2, Vec4},
    reflect::Reflect,
    render::{
        mesh::{Indices, Mesh},
        render_resource::{FilterMode, PrimitiveTopology},
    },
    utils::{hashbrown::hash_map::Entry, HashMap},
};

use crate::{
    math::{aabb::Aabb2d, extension::F32Integerize},
    tilemap::{
        coordinates,
        map::{TilemapRotation, TilemapTexture, TilemapTextureDescriptor},
    },
    utils::asset::AssetPath,
};

use super::{
    components::{TiledLoader, TiledUnloader},
    sprite::{SpriteUniform, TiledSpriteMaterial},
    xml::{
        layer::TiledLayer,
        tileset::{TiledTile, TiledTileset},
        MapOrientation, TiledGroup, TiledTilemap,
    },
};

/// Configuration for loading tiled tilemaps.
#[derive(Resource, Default, Reflect)]
pub struct TiledLoadConfig {
    pub map_path: Vec<String>,
    pub ignore_unregisterd_objects: bool,
}

#[derive(Debug, Clone, Reflect)]
pub struct PackedTiledTilemap {
    pub name: String,
    pub path: PathBuf,
    pub xml: TiledTilemap,
}

#[derive(Debug, Clone, Reflect)]
pub struct PackedTiledTileset {
    pub name: String,
    pub xml: TiledTileset,
    pub special_tiles: HashMap<u32, TiledTile>,
    pub texture: TilemapTexture,
}

/// A resource that manages tiled tilemaps.
/// 
/// You can load/unload tiled tilemaps using this resource.
#[derive(Resource, Default, Reflect)]
pub struct TiledTilemapManger {
    pub(crate) version: u32,
    pub(crate) cache: HashMap<String, PackedTiledTilemap>,
    pub(crate) loaded_levels: HashMap<String, Entity>,
}

impl TiledTilemapManger {
    pub fn reload_xml(&mut self, config: &TiledLoadConfig) {
        self.version += 1;
        self.cache = config
            .map_path
            .iter()
            .map(|path| {
                let path = Path::new(path);
                let name = path.file_stem().unwrap().to_str().unwrap().to_string();
                (
                    name.clone(),
                    PackedTiledTilemap {
                        name,
                        path: path.to_path_buf(),
                        xml: quick_xml::de::from_str(
                            &std::fs::read_to_string(path).unwrap_or_else(|err| {
                                panic!("Failed to read {:?}\n{:?}", path, err)
                            }),
                        )
                        .unwrap_or_else(|err| panic!("Failed to parse {:?}\n{:?}", path, err)),
                    },
                )
            })
            .collect();
    }

    pub fn load(&mut self, commands: &mut Commands, map_name: String, trans_ovrd: Option<Vec2>) {
        self.check_initialized();
        if self.loaded_levels.contains_key(&map_name) {
            error!("Trying to load {:?} that is already loaded!", map_name);
        } else {
            let entity = commands.spawn(TiledLoader {
                map: map_name.clone(),
                trans_ovrd,
            });
            self.loaded_levels.insert(map_name.clone(), entity.id());
        }
    }

    pub fn switch_to(&mut self, commands: &mut Commands, level: String, trans_ovrd: Option<Vec2>) {
        self.check_initialized();
        if self.loaded_levels.contains_key(&level.to_string()) {
            error!("Trying to load {:?} that is already loaded!", level);
        } else {
            self.unload_all(commands);
            self.load(commands, level, trans_ovrd);
        }
    }

    pub fn unload(&mut self, commands: &mut Commands, level: String) {
        let level = level.to_string();
        if let Some(l) = self.loaded_levels.get(&level) {
            commands.entity(*l).insert(TiledUnloader);
            self.loaded_levels.remove(&level);
        } else {
            error!("Trying to unload {:?} that is not loaded!", level);
        }
    }

    pub fn unload_all(&mut self, commands: &mut Commands) {
        for (_, l) in self.loaded_levels.iter() {
            commands.entity(*l).insert(TiledUnloader);
        }
        self.loaded_levels.clear();
    }

    #[inline]
    pub fn get_cached_data(&self) -> &HashMap<String, PackedTiledTilemap> {
        &self.cache
    }

    #[inline]
    pub fn is_loaded(&self, map_name: String) -> bool {
        self.loaded_levels.contains_key(&map_name)
    }

    #[inline]
    pub fn is_initialized(&self) -> bool {
        !self.cache.is_empty()
    }

    #[inline]
    fn check_initialized(&self) {
        assert_ne!(self.version, 0, "TiledTilemapManager is not initialized!");
    }
}

/// All the resources that are loaded from tiled tilemaps.
/// 
/// This includes tilesets, image meshes/materials, object meshes/materials, etc.
#[derive(Resource, Default, Reflect)]
pub struct TiledAssets {
    pub(crate) version: u32,
    pub(crate) tilesets: Vec<PackedTiledTileset>,
    /// (tileset_index, first_gid)
    pub(crate) tilemap_tilesets: HashMap<String, Vec<(usize, u32)>>,
    /// (mesh_handle, z)
    #[reflect(ignore)]
    pub(crate) image_layer_mesh: HashMap<String, HashMap<u32, (Handle<Mesh>, f32)>>,
    pub(crate) image_layer_materials: HashMap<String, HashMap<u32, Handle<TiledSpriteMaterial>>>,
    /// (mesh_handle, z)
    #[reflect(ignore)]
    pub(crate) object_mesh: HashMap<String, HashMap<u32, Handle<Mesh>>>,
    pub(crate) object_materials: HashMap<String, HashMap<u32, Handle<TiledSpriteMaterial>>>,
    pub(crate) object_z_order: HashMap<String, HashMap<u32, f32>>,
}

impl TiledAssets {
    /// Returns (tileset, first_gid)
    pub fn get_tileset(&self, gid: u32, tilemap: &str) -> (&PackedTiledTileset, u32) {
        let (index, first_gid) = self.tilemap_tilesets[tilemap]
            .iter()
            .rev()
            .find(|(_, first_gid)| *first_gid <= gid)
            .unwrap();
        (&self.tilesets[*index], *first_gid)
    }

    pub fn clone_image_layer_mesh_handle(&self, map: &str, layer: u32) -> (Handle<Mesh>, f32) {
        self.image_layer_mesh
            .get(map)
            .and_then(|meshes| meshes.get(&layer))
            .cloned()
            .unwrap()
    }

    pub fn clone_image_layer_material_handle(
        &self,
        map: &str,
        layer: u32,
    ) -> Handle<TiledSpriteMaterial> {
        self.image_layer_materials
            .get(map)
            .and_then(|materials| materials.get(&layer))
            .cloned()
            .unwrap()
    }

    pub fn clone_object_mesh_handle(&self, map: &str, object: u32) -> Handle<Mesh> {
        self.object_mesh
            .get(map)
            .and_then(|meshes| meshes.get(&object))
            .cloned()
            .unwrap()
    }

    pub fn get_object_z_order(&self, map: &str, object: u32) -> f32 {
        self.object_z_order
            .get(map)
            .and_then(|z_orders| z_orders.get(&object))
            .cloned()
            .unwrap()
    }

    pub fn clone_object_material_handle(
        &self,
        map: &str,
        object: u32,
    ) -> Handle<TiledSpriteMaterial> {
        self.object_materials
            .get(map)
            .and_then(|materials| materials.get(&object))
            .cloned()
            .unwrap()
    }

    pub fn initialize(
        &mut self,
        manager: &TiledTilemapManger,
        _config: &TiledLoadConfig,
        asset_server: &AssetServer,
        material_assets: &mut Assets<TiledSpriteMaterial>,
        mesh_assets: &mut Assets<Mesh>,
    ) {
        if self.version == manager.version {
            return;
        }

        self.version = manager.version;
        self.load_tilesets(manager, asset_server);
        self.load_map_assets(manager, asset_server, material_assets, mesh_assets);
    }

    fn load_tilesets(&mut self, manager: &TiledTilemapManger, asset_server: &AssetServer) {
        let tiled_xml = manager.get_cached_data();
        let mut tileset_records = HashMap::default();

        tiled_xml.iter().for_each(|(_, map)| {
            map.xml.tilesets.iter().for_each(|tileset_def| {
                let tileset_path = map.path.parent().unwrap().join(&tileset_def.source);
                let tileset_xml = quick_xml::de::from_str::<TiledTileset>(
                    &std::fs::read_to_string(&tileset_path).unwrap(),
                )
                .unwrap();

                match tileset_records.entry(tileset_xml.name.clone()) {
                    Entry::Occupied(e) => {
                        self.tilemap_tilesets
                            .entry(map.name.clone())
                            .or_default()
                            .push((*e.get(), tileset_def.first_gid));
                    }
                    Entry::Vacant(_) => {
                        self.tilemap_tilesets
                            .entry(map.name.clone())
                            .or_default()
                            .push((self.tilesets.len(), tileset_def.first_gid));
                    }
                }

                let source_path = tileset_path
                    .parent()
                    .unwrap()
                    .join(&tileset_xml.image.source);
                let texture = TilemapTexture {
                    texture: asset_server.load(source_path.to_asset_path()),
                    desc: TilemapTextureDescriptor {
                        size: UVec2 {
                            x: tileset_xml.image.width,
                            y: tileset_xml.image.height,
                        },
                        tile_size: UVec2 {
                            x: tileset_xml.tile_width,
                            y: tileset_xml.tile_height,
                        },
                        filter_mode: FilterMode::Nearest,
                    },
                    rotation: TilemapRotation::None,
                };

                self.tilesets.push(PackedTiledTileset {
                    name: tileset_xml.name.clone(),
                    special_tiles: tileset_xml
                        .special_tiles
                        .iter()
                        .map(|tile| (tile.id, tile.clone()))
                        .collect(),
                    xml: tileset_xml,
                    texture,
                });
            });

            self.tilemap_tilesets.values_mut().for_each(|v| {
                v.sort_by(|(_, a), (_, b)| a.cmp(b));
            });
        });
    }

    fn load_map_assets(
        &mut self,
        manager: &TiledTilemapManger,
        asset_server: &AssetServer,
        material_assets: &mut Assets<TiledSpriteMaterial>,
        mesh_assets: &mut Assets<Mesh>,
    ) {
        manager.get_cached_data().iter().for_each(|(_, map)| {
            self.load_layers(map, asset_server, material_assets, mesh_assets);
        });
    }

    fn load_layers(
        &mut self,
        map: &PackedTiledTilemap,
        asset_server: &AssetServer,
        material_assets: &mut Assets<TiledSpriteMaterial>,
        mesh_assets: &mut Assets<Mesh>,
    ) {
        self.load_image_layers(
            map,
            &map.xml.layers,
            asset_server,
            material_assets,
            mesh_assets,
        );
        self.load_objects(map, &map.xml.layers, material_assets, mesh_assets);

        self.load_groups(
            map,
            &map.xml.groups,
            asset_server,
            material_assets,
            mesh_assets,
        );
    }

    fn load_groups(
        &mut self,
        map: &PackedTiledTilemap,
        groups: &Vec<TiledGroup>,
        asset_server: &AssetServer,
        material_assets: &mut Assets<TiledSpriteMaterial>,
        mesh_assets: &mut Assets<Mesh>,
    ) {
        groups.iter().for_each(|group| {
            self.load_image_layers(
                map,
                &group.layers,
                asset_server,
                material_assets,
                mesh_assets,
            );
            self.load_objects(map, &group.layers, material_assets, mesh_assets);
            self.load_groups(
                map,
                &group.groups,
                asset_server,
                material_assets,
                mesh_assets,
            );
        });
    }

    fn load_image_layers(
        &mut self,
        map: &PackedTiledTilemap,
        layers: &Vec<TiledLayer>,
        asset_server: &AssetServer,
        material_assets: &mut Assets<TiledSpriteMaterial>,
        mesh_assets: &mut Assets<Mesh>,
    ) {
        layers
            .iter()
            .enumerate()
            .filter_map(|(z, layer)| {
                if let TiledLayer::Image(layer) = layer {
                    Some((z, layer))
                } else {
                    None
                }
            })
            .for_each(|(z, layer)| {
                let image_path = map
                    .path
                    .parent()
                    .unwrap()
                    .join(&layer.image.source)
                    .to_asset_path();
                let image = asset_server.load(image_path);
                self.image_layer_materials
                    .entry(map.name.clone())
                    .or_default()
                    .insert(
                        layer.id,
                        material_assets.add(TiledSpriteMaterial {
                            image: image.clone(),
                            data: SpriteUniform {
                                atlas: Aabb2d {
                                    min: Vec2::ZERO,
                                    max: Vec2::ONE,
                                },
                                tint: Vec4::new(
                                    layer.tint.r,
                                    layer.tint.g,
                                    layer.tint.b,
                                    layer.tint.a * layer.opacity,
                                ),
                            },
                        }),
                    );

                let image_size = Vec2::new(layer.image.width as f32, layer.image.height as f32);
                let image_verts = vec![
                    Vec2::ZERO,
                    Vec2::new(image_size.x, 0.),
                    Vec2::new(image_size.x, -image_size.y),
                    Vec2::new(0., -image_size.y),
                ];
                let image_uvs = vec![Vec2::ZERO, Vec2::X, Vec2::ONE, Vec2::Y];
                let tile_size = Vec2::new(map.xml.tile_width as f32, map.xml.tile_height as f32);
                let map_size = match map.xml.orientation {
                    MapOrientation::Orthogonal | MapOrientation::Isometric => {
                        coordinates::calculate_map_size(
                            UVec2::new(map.xml.width, map.xml.height),
                            tile_size,
                            map.xml.orientation.as_tilemap_type(map.xml.hex_side_length),
                        )
                    }
                    MapOrientation::Staggered | MapOrientation::Hexagonal => {
                        coordinates::calculate_map_size_staggered(
                            UVec2::new(map.xml.width, map.xml.height),
                            tile_size,
                            map.xml.hex_side_length,
                        )
                    }
                };

                let map_origin = match map.xml.orientation {
                    MapOrientation::Isometric => {
                        Vec2::new(-(map.xml.height as f32) / 2. * tile_size.x, 0.)
                    }
                    _ => Vec2::ZERO,
                };
                let map_area = Aabb2d {
                    min: Vec2::new(map_origin.x, map_origin.y - map_size.y),
                    max: Vec2::new(map_origin.x + map_size.x, map_origin.y - map_origin.y),
                };
                let origin = Vec2::new(layer.offset_x, -layer.offset_y) + map_origin;
                let unit_indices = vec![0, 3, 1, 1, 3, 2];

                let mut vertices =
                    vec![image_verts.iter().map(|v| *v + origin).collect::<Vec<_>>()];
                let mut uvs = vec![image_uvs.clone()];
                let mut indices = vec![unit_indices.clone()];
                let mut mesh_index = 0;

                if (layer.repeat_x || layer.repeat_y)
                    && (layer.offset_x < 0. || layer.offset_y < 0.)
                {
                    warn!(
                        "Repeated image layers must have positive offset! \
                        But got {} in layer {} in map {}! \
                        This will lead to wrong image repeating counts! \
                        But if you don't mind getting extra images, \
                        you can ignore this warning.",
                        origin - map_origin,
                        layer.name,
                        map.name
                    );
                }

                if layer.repeat_x {
                    vertices.clear();
                    uvs.clear();
                    indices.clear();

                    let left = ((origin.x - map_area.min.x) / image_size.x).ceil_to_u32();
                    let right = ((map_area.max.x - origin.x) / image_size.x).ceil_to_u32();
                    let repeat_origin_x = origin.x - left as f32 * image_size.x;
                    for i in 0..(left + right) {
                        let unclipped_uvs = image_uvs.clone();
                        let unclipped_verts = image_verts
                            .iter()
                            .map(|v| *v + Vec2::new(i as f32 * image_size.x + repeat_origin_x, 0.))
                            .collect();

                        uvs.push(unclipped_uvs);
                        vertices.push(unclipped_verts);
                        indices.push(unit_indices.iter().map(|i| i + mesh_index * 4).collect());
                        mesh_index += 1;
                    }
                }

                if layer.repeat_y {
                    let origin_images = vertices.clone();
                    vertices.clear();
                    uvs.clear();

                    let up = ((map_area.max.y - origin.y) / image_size.y).ceil_to_u32();
                    let down = ((origin.y - map_area.min.y) / image_size.y).ceil_to_u32();
                    let repeat_origin_y = origin.y - (down as f32 - 1.) * image_size.y;
                    for i in 0..(up + down) {
                        origin_images.iter().for_each(|image| {
                            let unclipped_uvs = image_uvs.clone();
                            let unclipped_verts = image
                                .iter()
                                .map(|v| {
                                    *v + Vec2::new(0., i as f32 * image_size.y + repeat_origin_y)
                                })
                                .collect();

                            uvs.push(unclipped_uvs);
                            vertices.push(unclipped_verts);
                            indices.push(unit_indices.iter().map(|i| i + mesh_index * 4).collect());
                            mesh_index += 1;
                        });
                    }
                }

                let mesh = mesh_assets.add(
                    Mesh::new(PrimitiveTopology::TriangleList)
                        .with_inserted_attribute(
                            Mesh::ATTRIBUTE_POSITION,
                            vertices
                                .into_iter()
                                .flat_map(|image| image.into_iter().map(|v| v.extend(0.)))
                                .collect::<Vec<_>>(),
                        )
                        .with_inserted_attribute(
                            Mesh::ATTRIBUTE_UV_0,
                            uvs.into_iter()
                                .flat_map(|image| image.into_iter())
                                .collect::<Vec<_>>(),
                        )
                        .with_indices(Some(Indices::U16(
                            indices
                                .into_iter()
                                .flat_map(|image| image.into_iter())
                                .collect::<Vec<_>>(),
                        ))),
                );

                self.image_layer_mesh
                    .entry(map.name.clone())
                    .or_default()
                    .insert(layer.id, (mesh, z as f32));
            });
    }

    fn load_objects(
        &mut self,
        map: &PackedTiledTilemap,
        layers: &Vec<TiledLayer>,
        material_assets: &mut Assets<TiledSpriteMaterial>,
        mesh_assets: &mut Assets<Mesh>,
    ) {
        let mut objects = layers
            .iter()
            .enumerate()
            .filter_map(|(z, layer)| {
                if let TiledLayer::Objects(obj_layer) = layer {
                    Some((z, obj_layer))
                } else {
                    None
                }
            })
            .flat_map(|(z, layer)| {
                let tint = Vec4::new(
                    layer.tint.r,
                    layer.tint.g,
                    layer.tint.b,
                    layer.tint.a * layer.opacity,
                );

                layer
                    .objects
                    .iter()
                    .enumerate()
                    .for_each(|(obj_z, object)| {
                        let obj_z = obj_z as f32 / layer.objects.len() as f32 + z as f32;
                        self.object_z_order
                            .entry(map.name.clone())
                            .or_default()
                            .insert(object.id, obj_z);
                    });
                    
                layer
                    .objects
                    .iter()
                    .filter_map(move |obj| obj.gid.map(|_| (obj, tint)))
            })
            .collect::<Vec<_>>();

        objects.sort_by(|(lhs, _), (rhs, _)| lhs.y.partial_cmp(&rhs.y).unwrap());

        let mesh_ext = objects
            .iter()
            .map(|(object, _)| {
                let flipping = object.gid.unwrap_or_default() >> 30;
                let mesh = Mesh::new(PrimitiveTopology::TriangleList)
                    .with_inserted_attribute(
                        Mesh::ATTRIBUTE_POSITION,
                        vec![
                            Vec2::new(0., object.height),
                            Vec2::new(object.width, object.height),
                            Vec2::new(object.width, 0.),
                            Vec2::ZERO,
                        ]
                        .into_iter()
                        .map(|v| {
                            Vec2::from_angle(-object.rotation / 180. * PI)
                                .rotate(v)
                                .extend(0.)
                        })
                        .collect::<Vec<_>>(),
                    )
                    .with_inserted_attribute(
                        Mesh::ATTRIBUTE_UV_0,
                        vec![Vec2::ZERO, Vec2::X, Vec2::ONE, Vec2::Y]
                            .into_iter()
                            .map(|mut v| {
                                if flipping & (1 << 1) != 0 {
                                    v.x = 1. - v.x;
                                }
                                if flipping & (1 << 0) != 0 {
                                    v.y = 1. - v.y;
                                }
                                v
                            })
                            .collect::<Vec<_>>(),
                    )
                    .with_indices(Some(Indices::U16(vec![2, 0, 1, 3, 0, 2])));

                (object.id, mesh_assets.add(mesh))
            })
            .collect::<Vec<_>>();

        self.object_mesh
            .entry(map.name.clone())
            .or_default()
            .extend(mesh_ext);

        let mat_ext = objects
            .iter()
            .map(|(object, tint)| {
                let gid = object.gid.unwrap() & 0x3FFF_FFFF;
                let (tileset, first_gid) = &self.get_tileset(gid, &map.name);
                (
                    object.id,
                    material_assets.add(TiledSpriteMaterial {
                        image: tileset.texture.texture.clone(),
                        data: SpriteUniform {
                            atlas: tileset.texture.get_atlas_rect(gid - first_gid),
                            tint: (*tint).into(),
                        },
                    }),
                )
            })
            .collect::<Vec<_>>();

        self.object_materials
            .entry(map.name.clone())
            .or_default()
            .extend(mat_ext);
    }
}
