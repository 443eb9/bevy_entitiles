use std::{f32::consts::PI, path::PathBuf};

use bevy::{
    asset::{io::Reader, Asset, AssetId, AssetLoader, AssetServer, Assets, Handle, LoadContext},
    ecs::{entity::Entity, system::Resource},
    log::{error, warn},
    math::{Rect, UVec2, Vec2, Vec4},
    prelude::{Deref, EventWriter},
    reflect::Reflect,
    render::{
        mesh::{Indices, Mesh},
        render_asset::RenderAssetUsages,
        render_resource::{FilterMode, PrimitiveTopology},
    },
    utils::HashMap,
};
use futures_lite::AsyncReadExt;
use thiserror::Error;

use crate::{
    math::ext::F32Integerize,
    tiled::{
        events::{TiledMapEvent, TiledMapUnloader},
        sprite::{SpriteUniform, TiledSpriteMaterial},
        xml::{
            layer::TiledLayer, property::Components, tileset::TiledTileset, MapOrientation,
            TiledGroup, TiledXml,
        },
    },
    tilemap::{
        coordinates,
        map::{TilemapAnimations, TilemapTexture, TilemapTextureDescriptor, TilemapTextures},
        tile::{RawTileAnimation, TileAnimation},
    },
    utils::asset::AssetPath,
};

/// Configuration for loading tiled tilemaps.
#[derive(Resource, Default, Reflect)]
pub struct TiledLoadConfig {
    pub z_index: f32,
    pub ignore_unregisterd_objects: bool,
    pub ignore_unregisterd_custom_tiles: bool,
}

#[derive(Asset, Debug, Clone, Reflect)]
pub struct PackedTiledTilemap {
    pub name: String,
    pub path: PathBuf,
    pub xml: TiledXml,
    pub tilesets: HashMap<String, Handle<TiledTileset>>,
}

#[derive(Error, Debug)]
pub enum TiledTilemapLoaderError {
    #[error("Io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Xml error: {0}")]
    Xml(#[from] quick_xml::DeError),
}

#[derive(Default)]
pub struct TiledTilemapLoader;

impl AssetLoader for TiledTilemapLoader {
    type Asset = PackedTiledTilemap;

    type Settings = ();

    type Error = TiledTilemapLoaderError;

    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a Self::Settings,
        load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut buf = String::new();
        reader.read_to_string(&mut buf).await?;
        let path = load_context.path().to_path_buf();
        let xml = quick_xml::de::from_str::<TiledXml>(&buf)?;

        Ok(PackedTiledTilemap {
            name: path.file_name().unwrap().to_str().unwrap().to_string(),
            tilesets: xml
                .tilesets
                .iter()
                .map(|ts| {
                    (
                        ts.source.clone(),
                        load_context.load(path.parent().unwrap().join(&ts.source)),
                    )
                })
                .collect(),
            path,
            xml,
        })
    }
}

#[derive(Error, Debug)]
pub enum TiledTilesetLoaderError {
    #[error("Io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Xml error: {0}")]
    Xml(#[from] quick_xml::DeError),
}

#[derive(Default)]
pub struct TiledTilesetLoader;

impl AssetLoader for TiledTilesetLoader {
    type Asset = TiledTileset;

    type Settings = ();

    type Error = TiledTilemapLoaderError;

    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a Self::Settings,
        _load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut buf = String::new();
        reader.read_to_string(&mut buf).await?;
        quick_xml::de::from_str(&buf).map_err(Into::into)
    }
}

#[derive(Debug, Clone, Reflect)]
pub struct TiledCustomTileInstance {
    pub properties: Components,
    pub ty: String,
}

#[derive(Debug, Clone, Reflect)]
pub struct PackedTiledTileset {
    pub name: String,
    pub texture: TilemapTexture,
    pub animated_tiles: HashMap<u32, TileAnimation>,
    pub custom_properties_tiles: HashMap<u32, TiledCustomTileInstance>,
}

#[derive(Resource, Default, Reflect, Deref)]
pub struct TiledLoadedMaps(pub(crate) HashMap<AssetId<PackedTiledTilemap>, Entity>);

impl TiledLoadedMaps {
    /// Quick utility to unload all maps.
    pub fn unload_all(&self, event_writer: &mut EventWriter<TiledMapEvent>) {
        event_writer.send_batch(
            self.keys()
                .map(|id| TiledMapEvent::Unload(TiledMapUnloader { map: *id })),
        );
    }
}

#[derive(Clone, Copy, Reflect)]
pub struct TiledTilesetMeta {
    pub global_tileset_index: usize,
    pub texture_index: u32,
    pub first_gid: u32,
}

#[derive(Resource, Default, Reflect, Deref)]
pub struct TiledTilemapToAssets(
    pub(crate) HashMap<AssetId<PackedTiledTilemap>, Handle<TiledAssets>>,
);

/// All the resources that are loaded from tiled tilemaps.
///
/// This includes tilesets, image meshes/materials, object meshes/materials, etc.
#[derive(Asset, Default, Reflect)]
pub struct TiledAssets {
    /// Used when spawning objects that use a tile as texture.
    pub(crate) tilesets: Vec<PackedTiledTileset>,
    pub(crate) tileset_metas: Vec<TiledTilesetMeta>,
    pub(crate) tilemap_data: (Handle<TilemapTextures>, TilemapAnimations),
    /// (mesh_handle, z)
    #[reflect(ignore)]
    pub(crate) image_layer_mesh: HashMap<u32, (Handle<Mesh>, f32)>,
    pub(crate) image_layer_materials: HashMap<u32, Handle<TiledSpriteMaterial>>,
    #[reflect(ignore)]
    pub(crate) object_mesh: HashMap<u32, Handle<Mesh>>,
    pub(crate) object_materials: HashMap<u32, Handle<TiledSpriteMaterial>>,
    pub(crate) object_z_order: HashMap<u32, f32>,
}

impl TiledAssets {
    pub fn get_tileset(&self, tile_id: u32) -> (&PackedTiledTileset, TiledTilesetMeta) {
        let meta = self.get_tileset_meta(tile_id);
        (self.tilesets.get(meta.global_tileset_index).unwrap(), meta)
    }

    pub fn get_tileset_meta(&self, tile_id: u32) -> TiledTilesetMeta {
        *self
            .tileset_metas
            .iter()
            .rev()
            .find(|meta| meta.first_gid <= tile_id)
            .unwrap()
    }

    pub fn get_tilemap_data(&self) -> (Handle<TilemapTextures>, TilemapAnimations) {
        self.tilemap_data.clone()
    }

    pub fn clone_image_layer_mesh_handle(&self, layer: u32) -> (Handle<Mesh>, f32) {
        self.image_layer_mesh.get(&layer).cloned().unwrap()
    }

    pub fn clone_image_layer_material_handle(&self, layer: u32) -> Handle<TiledSpriteMaterial> {
        self.image_layer_materials.get(&layer).cloned().unwrap()
    }

    pub fn clone_object_mesh_handle(&self, object: u32) -> Handle<Mesh> {
        self.object_mesh.get(&object).cloned().unwrap()
    }

    pub fn get_object_z_order(&self, object: u32) -> f32 {
        self.object_z_order.get(&object).cloned().unwrap()
    }

    pub fn clone_object_material_handle(&self, object: u32) -> Handle<TiledSpriteMaterial> {
        self.object_materials.get(&object).cloned().unwrap()
    }

    pub fn new(
        map: &PackedTiledTilemap,
        asset_server: &AssetServer,
        tileset_assets: &Assets<TiledTileset>,
        material_assets: &mut Assets<TiledSpriteMaterial>,
        textures_assets: &mut Assets<TilemapTextures>,
        mesh_assets: &mut Assets<Mesh>,
    ) -> Self {
        let mut instance = Self::default();
        instance.load_tilesets(map, asset_server, textures_assets, tileset_assets);
        instance.load_layers(map, asset_server, material_assets, mesh_assets);
        instance
    }

    fn load_tilesets(
        &mut self,
        map: &PackedTiledTilemap,
        asset_server: &AssetServer,
        textures_assets: &mut Assets<TilemapTextures>,
        tileset_assets: &Assets<TiledTileset>,
    ) {
        let mut tilesets_records = HashMap::default();

        // Load all tilesets
        let mut animations = TilemapAnimations::default();
        let mut textures = Vec::with_capacity(map.xml.tilesets.len());
        let mut metas = Vec::with_capacity(map.xml.tilesets.len());

        for (texture_index, tileset_def) in map.xml.tilesets.iter().enumerate() {
            let mut animated_tiles = HashMap::default();
            let mut custom_properties_tiles = HashMap::default();

            let tileset_path = map.path.parent().unwrap().join(&tileset_def.source);
            let Some(tileset_xml) =
                tileset_assets.get(map.tilesets.get(&tileset_def.source).unwrap())
            else {
                return;
            };

            if let Some(global_tileset_index) = tilesets_records.get(&tileset_xml.name).cloned() {
                metas.push(TiledTilesetMeta {
                    global_tileset_index,
                    texture_index: texture_index as u32,
                    first_gid: tileset_def.first_gid,
                });
                textures.push(self.tilesets[global_tileset_index].texture.clone());
                continue;
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
                },
            };

            tileset_xml
                .special_tiles
                .iter()
                .map(|tile| (tile.id, tile.clone()))
                .for_each(|(atlas_index, tile)| {
                    // Animated tiles
                    if let Some(tiled_animation) = tile.animation {
                        let frames = tiled_animation.frames;
                        let anim = animations.register(RawTileAnimation {
                            // TODO maybe support?
                            fps: (1000. / frames[0].duration as f32) as u32,
                            sequence: frames
                                .into_iter()
                                .map(|frame| (texture_index as u32, frame.tile_id))
                                .collect(),
                        });
                        animated_tiles.insert(atlas_index, anim);
                    }
                    // Tiles with custom properties
                    if !tile.ty.is_empty() {
                        custom_properties_tiles.insert(
                            atlas_index,
                            TiledCustomTileInstance {
                                properties: tile.properties.unwrap_or_default(),
                                ty: tile.ty,
                            },
                        );
                    }
                });

            tilesets_records.insert(tileset_xml.name.clone(), self.tilesets.len());
            metas.push(TiledTilesetMeta {
                global_tileset_index: self.tilesets.len(),
                texture_index: texture_index as u32,
                first_gid: tileset_def.first_gid,
            });
            self.tilesets.push(PackedTiledTileset {
                name: tileset_xml.name.clone(),
                texture: texture.clone(),
                animated_tiles,
                custom_properties_tiles,
            });
            textures.push(texture);
        }

        self.tileset_metas = metas;

        self.tilemap_data = (
            textures_assets.add(TilemapTextures::new(textures, FilterMode::Nearest)),
            animations,
        );
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
        self.load_objects(&map.xml.layers, material_assets, mesh_assets);

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
            self.load_objects(&group.layers, material_assets, mesh_assets);
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
                self.image_layer_materials.insert(
                    layer.id,
                    material_assets.add(TiledSpriteMaterial {
                        image: image.clone(),
                        data: SpriteUniform {
                            atlas: [0., 0., 1., 1.].into(),
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
                let map_area = Rect {
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
                    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::all())
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
                        .with_inserted_indices(Indices::U16(
                            indices
                                .into_iter()
                                .flat_map(|image| image.into_iter())
                                .collect::<Vec<_>>(),
                        )),
                );

                self.image_layer_mesh.insert(layer.id, (mesh, z as f32));
            });
    }

    fn load_objects(
        &mut self,
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
                        self.object_z_order.insert(object.id, obj_z);
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

                if object.rotation >= 0.001 {
                    warn!(
                        "Object rotation is not recommended! It will results in wrong collision shapes. \
                        But if you just want to use this object as sort of static image it will be ok."
                    );
                }

                let mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::all())
                    .with_inserted_attribute(
                        Mesh::ATTRIBUTE_POSITION,
                        vec![
                            Vec2::new(0., object.height),
                            Vec2::new(object.width, object.height),
                            Vec2::new(object.width, 0.),
                            Vec2::ZERO,
                        ]
                        .into_iter()
                        .map(|v| Vec2::from_angle(-object.rotation / 180. * PI).rotate(v))
                        .map(|v| v - Vec2::new(object.width / 2., -object.height / 2.))
                        .map(|v| v.extend(0.))
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
                    .with_inserted_indices(Indices::U16(vec![2, 0, 1, 3, 0, 2]));

                (object.id, mesh_assets.add(mesh))
            })
            .collect::<Vec<_>>();

        self.object_mesh.extend(mesh_ext);

        let mat_ext = objects
            .iter()
            .map(|(object, tint)| {
                let gid = object.gid.unwrap() & 0x3FFF_FFFF;
                let (tileset, meta) = &self.get_tileset(gid);
                // TODO support animation and flipping
                let aabb = tileset.texture.get_atlas_rect(gid - meta.first_gid);
                (
                    object.id,
                    material_assets.add(TiledSpriteMaterial {
                        image: tileset.texture.texture.clone(),
                        data: SpriteUniform {
                            atlas: [aabb.min.x, aabb.min.y, aabb.max.x, aabb.max.y].into(),
                            tint: (*tint).into(),
                        },
                    }),
                )
            })
            .collect::<Vec<_>>();

        self.object_materials.extend(mat_ext);
    }
}
