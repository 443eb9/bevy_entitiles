use std::path::{Path, PathBuf};

use bevy::{
    asset::{Asset, AssetServer, Assets, Handle},
    ecs::{
        entity::Entity,
        system::{Commands, Resource},
    },
    log::error,
    math::{UVec2, Vec2},
    reflect::Reflect,
    render::{
        mesh::{Indices, Mesh},
        render_resource::{FilterMode, PrimitiveTopology},
        texture::Image,
    },
    sprite::Mesh2d,
    utils::{HashMap, HashSet},
};

use crate::{
    math::extension::F32Integerize,
    tilemap::map::{TilemapRotation, TilemapTexture, TilemapTextureDescriptor},
    utils::asset::AssetPath,
};

use super::{
    components::{TiledLoader, TiledUnloader},
    sprite::TiledSpriteMaterial,
    xml::{
        layer::{ImageLayer, TiledLayer},
        tileset::TiledTileset,
        TiledTilemap, TilesetDef,
    },
};

#[derive(Resource, Default, Reflect)]
pub struct TiledLoadConfig {
    pub map_path: Vec<String>,
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
    pub def: TilesetDef,
    pub xml: TiledTileset,
    pub texture: TilemapTexture,
}

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
                        xml: quick_xml::de::from_str(&std::fs::read_to_string(path).unwrap())
                            .unwrap(),
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

#[derive(Resource, Default, Reflect)]
pub struct TiledAssets {
    pub(crate) version: u32,
    /// First gid to tileset
    pub(crate) tilesets: Vec<PackedTiledTileset>,
    #[reflect(ignore)]
    pub(crate) image_layer_mesh: HashMap<String, HashMap<u32, Handle<Mesh>>>,
    pub(crate) image_layer_materials: HashMap<String, HashMap<u32, Handle<TiledSpriteMaterial>>>,
}

impl TiledAssets {
    pub fn get_tileset(&self, gid: u32) -> &PackedTiledTileset {
        self.tilesets
            .iter()
            .rev()
            .find(|tileset| tileset.def.first_gid <= gid)
            .unwrap()
    }

    pub fn clone_image_layer_mesh_handle(&self, map: &str, layer: u32) -> Handle<Mesh> {
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
        self.load_image_layers(manager, asset_server, material_assets, mesh_assets);
    }

    fn load_tilesets(&mut self, manager: &TiledTilemapManger, asset_server: &AssetServer) {
        let tiled_xml = manager.get_cached_data();
        let tileset_records: HashSet<u32> = HashSet::default();

        self.tilesets = tiled_xml
            .iter()
            .flat_map(|(_, map)| {
                map.xml.tilesets.iter().filter_map(|tileset_def| {
                    if tileset_records.contains(&tileset_def.first_gid) {
                        return None;
                    }

                    let tileset_path = map.path.parent().unwrap().join(&tileset_def.source);
                    let tileset_xml = quick_xml::de::from_str::<TiledTileset>(
                        &std::fs::read_to_string(&tileset_path).unwrap(),
                    )
                    .unwrap();

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

                    Some(PackedTiledTileset {
                        name: tileset_xml.name.clone(),
                        def: tileset_def.clone(),
                        xml: tileset_xml,
                        texture,
                    })
                })
            })
            .collect::<Vec<_>>();

        self.tilesets
            .sort_by(|l, r| l.def.first_gid.cmp(&r.def.first_gid));
    }

    fn load_image_layers(
        &mut self,
        manager: &TiledTilemapManger,
        asset_server: &AssetServer,
        material_assets: &mut Assets<TiledSpriteMaterial>,
        mesh_assets: &mut Assets<Mesh>,
    ) {
        manager.get_cached_data().iter().for_each(|(_, map)| {
            map.xml
                .layers
                .iter()
                .rev()
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
                            }),
                        );

                    let origin = Vec2::new(layer.offset_x, layer.offset_y);
                    let image_size = Vec2::new(layer.image.width as f32, layer.image.height as f32);
                    let image_verts = vec![
                        Vec2::ZERO,
                        Vec2::new(0., image_size.y),
                        image_size,
                        Vec2::new(image_size.x, 0.),
                    ];
                    let image_uvs = vec![Vec2::Y, Vec2::ZERO, Vec2::X, Vec2::ONE];
                    let map_size = Vec2::new(
                        (map.xml.width * map.xml.tile_width) as f32,
                        (map.xml.height * map.xml.tile_height) as f32,
                    );
                    let unit_indices = vec![0, 1, 2, 0, 2, 3];

                    let mut vertices =
                        vec![image_verts.iter().map(|v| *v + origin).collect::<Vec<_>>()];
                    let mut uvs = vec![image_uvs.clone()];
                    let mut indices = vec![unit_indices.clone()];
                    let mut mesh_index = 0;

                    if layer.repeat_x {
                        println!("repeat_x");
                        vertices.clear();
                        uvs.clear();
                        indices.clear();

                        let left = (origin.x / image_size.x).ceil_to_u32();
                        let right = ((map_size.x - origin.x) / image_size.x).ceil_to_u32();
                        let repeat_origin_x = origin.x - left as f32 * image_size.x;
                        for i in 0..(left + right) {
                            uvs.push(image_uvs.clone());
                            vertices.push(
                                image_verts
                                    .iter()
                                    .map(|v| {
                                        *v + Vec2::new(
                                            i as f32 * image_size.x + repeat_origin_x,
                                            0.,
                                        )
                                    })
                                    .collect(),
                            );
                            indices.push(unit_indices.iter().map(|i| i + mesh_index * 4).collect());
                            mesh_index += 1;
                        }
                    }

                    if layer.repeat_y {
                        let origin_images = vertices.clone();
                        vertices.clear();
                        uvs.clear();

                        let up = ((map_size.y - origin.y) / image_size.y).ceil_to_u32();
                        let down = (origin.y / image_size.y).ceil_to_u32();
                        let repeat_origin_y = origin.y - down as f32 * image_size.y;
                        for i in 0..(up + down) {
                            origin_images.iter().for_each(|image| {
                                uvs.push(image_uvs.clone());
                                vertices.push(
                                    image
                                        .iter()
                                        .map(|v| {
                                            *v + Vec2::new(
                                                0.,
                                                i as f32 * image_size.y + repeat_origin_y,
                                            )
                                        })
                                        .collect(),
                                );
                                indices.push(
                                    unit_indices.iter().map(|i| i + mesh_index * 4).collect(),
                                );
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
                                    .flat_map(|image| image.into_iter().map(|v| v.extend(z as f32)))
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
                        .insert(layer.id, mesh);
                });
        });
    }
}
