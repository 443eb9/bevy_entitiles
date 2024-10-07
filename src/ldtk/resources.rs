use std::path::Path;

use bevy::{
    asset::{io::Reader, Asset, AssetId, AssetLoader, AssetServer, Assets, Handle, LoadContext},
    ecs::{entity::Entity, system::Resource},
    log::error,
    math::{IVec2, UVec2, Vec2},
    prelude::{Deref, DerefMut, EventWriter},
    reflect::Reflect,
    render::{
        mesh::{Indices, Mesh},
        render_asset::RenderAssetUsages,
        render_resource::{FilterMode, PrimitiveTopology},
    },
    sprite::{Mesh2dHandle, SpriteBundle, TextureAtlasLayout},
    utils::HashMap,
};
use futures_lite::AsyncReadExt;
use thiserror::Error;

use crate::{
    ldtk::{
        components::{EntityIid, LayerIid, LevelIid},
        json::{definitions::EntityDef, EntityRef, LdtkJson, TocInstance},
        sprite::{AtlasRect, LdtkEntityMaterial},
    },
    prelude::{LdtkLevel, LdtkLevelEvent, LdtkLevelUnloader},
    serializing::pattern::{PackedPatternLayers, PatternsLayer, TilemapPattern},
    tilemap::{
        map::{TilemapTexture, TilemapTextureDescriptor},
        tile::RawTileAnimation,
    },
};

/// All the patterns loaded from the LDtk file.
#[derive(Resource, Reflect, Default, Clone)]
pub struct LdtkPatterns {
    pub pattern_size: UVec2,
    #[reflect(ignore)]
    pub patterns: Vec<(
        Vec<Option<TilemapPattern>>,
        Option<TilemapTexture>,
        Option<LayerIid>,
    )>,
    #[reflect(ignore)]
    pub backgrounds: Vec<Option<SpriteBundle>>,
    pub idents: Vec<String>,
    pub idents_to_index: HashMap<String, usize>,
}

impl LdtkPatterns {
    #[inline]
    pub fn new(idents: Vec<String>, pattern_size: UVec2) -> Self {
        Self {
            idents_to_index: idents
                .iter()
                .enumerate()
                .map(|(i, s)| (s.clone(), i))
                .collect(),
            idents,
            pattern_size,
            ..Default::default()
        }
    }

    pub fn add_pattern(
        &mut self,
        layer_index: usize,
        layer_iid: &LayerIid,
        pattern: TilemapPattern,
        texture: &Option<TilemapTexture>,
        identifier: &str,
    ) {
        if layer_index >= self.patterns.len() {
            self.patterns.resize(layer_index + 1, (vec![], None, None));
        }
        let (layer, layer_texture, iid) = self.patterns.get_mut(layer_index).unwrap();
        let pattern_index = self.idents_to_index[identifier];

        if layer_texture.is_none() {
            *layer_texture = texture.clone();
        }
        if iid.is_none() {
            *iid = Some(layer_iid.clone());
        }

        if pattern_index >= layer.len() {
            layer.resize(pattern_index + 1, None);
        }
        layer[pattern_index] = Some(pattern);
    }

    pub fn add_background(&mut self, identifier: &str, background: SpriteBundle) {
        let pattern_index = self.idents_to_index[identifier];
        if pattern_index >= self.backgrounds.len() {
            self.backgrounds.resize(pattern_index + 1, None);
        }

        self.backgrounds[pattern_index] = Some(background);
    }

    /// Pack the patterns into a `PackedPatternLayers` for wfc.
    pub fn pack(&self) -> PackedPatternLayers {
        PackedPatternLayers::new(
            self.pattern_size,
            self.patterns
                .iter()
                .filter_map(|(layer, texture, iid)| {
                    iid.clone().map(|iid| {
                        PatternsLayer::new(
                            Some(iid.0),
                            self.pattern_size,
                            layer.iter().map(|p| p.clone().unwrap()).collect(),
                            texture.clone(),
                        )
                    })
                })
                .collect(),
        )
    }

    #[inline]
    pub fn is_ready(&self) -> bool {
        !self.patterns.is_empty() && !self.idents.is_empty()
    }
}

#[derive(Resource, Default, Deref)]
pub struct LdtkJsonToAssets(pub(crate) HashMap<AssetId<LdtkJson>, Handle<LdtkAssets>>);

#[derive(Resource, Default, Deref)]
pub struct LdtkLevelIdentifierToIid(
    pub(crate) HashMap<AssetId<LdtkJson>, HashMap<String, LevelIid>>,
);

/// All the tilemaps loaded from the LDtk file.
///
/// This includes tilesets, entity meshes/materials etc.
#[derive(Asset, Default, Reflect)]
pub struct LdtkAssets {
    /// tileset iid to texture
    pub(crate) tilesets: HashMap<i32, TilemapTexture>,
    /// tileset iid to texture atlas handle
    pub(crate) atlas_handles: HashMap<i32, Handle<TextureAtlasLayout>>,
    /// entity identifier to entity definition
    pub(crate) entity_defs: HashMap<String, EntityDef>,
    /// entity iid to mesh handle
    pub(crate) meshes: HashMap<String, Mesh2dHandle>,
    /// entity iid to material handle
    pub(crate) materials: HashMap<String, Handle<LdtkEntityMaterial>>,
}

impl LdtkAssets {
    pub fn get_tileset(&self, tileset_uid: i32) -> &TilemapTexture {
        self.tilesets.get(&tileset_uid).unwrap()
    }

    pub fn clone_atlas_handle(&self, tileset_uid: i32) -> Handle<TextureAtlasLayout> {
        self.atlas_handles.get(&tileset_uid).unwrap().clone()
    }

    pub fn get_entity_def(&self, identifier: &String) -> &EntityDef {
        self.entity_defs.get(identifier).unwrap()
    }

    pub fn clone_mesh_handle(&self, iid: &String) -> Mesh2dHandle {
        self.meshes.get(iid).unwrap().clone()
    }

    pub fn clone_material_handle(&self, iid: &String) -> Handle<LdtkEntityMaterial> {
        self.materials.get(iid).unwrap().clone()
    }

    /// Initialize the assets.
    ///
    /// You need to call this after you changed something like the size of an entity,
    /// or maybe the identifier of an entity.
    pub fn new(
        config: &LdtkLevelConfig,
        ldtk_data: &LdtkJson,
        asset_server: &AssetServer,
        atlas_layouts: &mut Assets<TextureAtlasLayout>,
        material_assets: &mut Assets<LdtkEntityMaterial>,
        mesh_assets: &mut Assets<Mesh>,
    ) -> Self {
        let mut instance = Self::default();
        instance.load_texture(config, ldtk_data, asset_server, atlas_layouts);
        instance.load_entities(config, ldtk_data, material_assets, mesh_assets);
        instance
    }

    fn load_texture(
        &mut self,
        config: &LdtkLevelConfig,
        ldtk_data: &LdtkJson,
        asset_server: &AssetServer,
        atlas_layouts: &mut Assets<TextureAtlasLayout>,
    ) {
        ldtk_data.defs.tilesets.iter().for_each(|tileset| {
            let Some(path) = tileset.rel_path.as_ref() else {
                return;
            };

            let texture = asset_server.load(Path::new(&config.asset_path_prefix).join(path));
            let desc = TilemapTextureDescriptor {
                size: UVec2 {
                    x: tileset.px_wid as u32,
                    y: tileset.px_hei as u32,
                },
                tile_size: UVec2 {
                    x: tileset.tile_grid_size as u32,
                    y: tileset.tile_grid_size as u32,
                },
            };
            let texture = TilemapTexture { texture, desc };

            self.tilesets.insert(tileset.uid, texture.clone());
            self.atlas_handles
                .insert(tileset.uid, atlas_layouts.add(texture.as_atlas_layout()));
        });
    }

    fn load_entities(
        &mut self,
        config: &LdtkLevelConfig,
        ldtk_data: &LdtkJson,
        material_assets: &mut Assets<LdtkEntityMaterial>,
        mesh_assets: &mut Assets<Mesh>,
    ) {
        ldtk_data.defs.entities.iter().for_each(|entity| {
            self.entity_defs
                .insert(entity.identifier.clone(), entity.clone());
        });

        ldtk_data
            .levels
            .iter()
            .map(|level| level.layer_instances.iter())
            .flatten()
            .map(|layer| layer.entity_instances.iter())
            .flatten()
            .for_each(|entity_instance| {
                let Some(tile_rect) = entity_instance.tile.as_ref() else {
                    return;
                };

                let texture_size = self.get_tileset(tile_rect.tileset_uid).desc.size.as_vec2();
                self.materials.insert(
                    entity_instance.iid.clone(),
                    material_assets.add(LdtkEntityMaterial {
                        texture: self.get_tileset(tile_rect.tileset_uid).texture.clone(),
                        atlas_rect: AtlasRect {
                            min: IVec2::new(tile_rect.x_pos, tile_rect.y_pos).as_vec2()
                                / texture_size,
                            max: IVec2::new(
                                tile_rect.x_pos + tile_rect.width,
                                tile_rect.y_pos + tile_rect.height,
                            )
                            .as_vec2()
                                / texture_size,
                        },
                    }),
                );

                let sprite_mesh = self.entity_defs[&entity_instance.identifier]
                    .tile_render_mode
                    .get_mesh(entity_instance, tile_rect, &self.entity_defs);

                let entity_depth = ldtk_data
                    .defs
                    .entities
                    .iter()
                    .enumerate()
                    .map(|(index, entity)| {
                        (
                            entity.identifier.clone(),
                            (ldtk_data.defs.entities.len() - index) as f32 + config.z_index as f32,
                        )
                    })
                    .collect::<HashMap<String, f32>>();

                let mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::all())
                    .with_inserted_attribute(
                        Mesh::ATTRIBUTE_POSITION,
                        sprite_mesh
                            .vertices
                            .into_iter()
                            .map(|p| p.extend(entity_depth[&entity_instance.identifier]))
                            .collect::<Vec<_>>(),
                    )
                    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, sprite_mesh.uvs)
                    .with_inserted_indices(Indices::U16(sprite_mesh.indices));
                self.meshes
                    .insert(entity_instance.iid.clone(), mesh_assets.add(mesh).into());
            });
    }
}

/// The wfc result of the LDtk file.
///
/// Use this to load your wfc result in the MultiMaps way.
#[cfg(feature = "algorithm")]
#[derive(Resource, Default, Reflect)]
pub struct LdtkWfcManager {
    pub(crate) wfc_data: Option<crate::algorithm::wfc::WfcData>,
    pub(crate) idents: Vec<String>,
    pub(crate) pattern_size: UVec2,
}

#[cfg(feature = "algorithm")]
impl LdtkWfcManager {
    /// Get the level identifier at the given level index.
    pub fn get_ident(&self, level_index: UVec2) -> Option<String> {
        let idx = self.wfc_data.as_ref()?.get(level_index)?;
        Some(self.idents[idx as usize].clone())
    }

    /// Calculate the translation of the given level index.
    pub fn get_translation(&self, level_index: IVec2, slot_size: Vec2) -> Vec2 {
        (level_index.as_vec2() + Vec2::Y) * self.pattern_size.as_vec2() * slot_size
    }
}

#[derive(Resource, Debug, Clone, Default)]
pub struct LdtkTocs(pub(crate) HashMap<String, HashMap<EntityRef, TocInstance>>);

impl LdtkTocs {
    pub fn get(&self, identifier: String, entity: EntityRef) -> Option<&TocInstance> {
        self.0.get(&identifier)?.get(&entity)
    }

    pub fn get_all(&self, identifier: String) -> Option<&HashMap<EntityRef, TocInstance>> {
        self.0.get(&identifier)
    }
}

/// The additional layers of the LDtk file.
///
/// This includes path layer and physics layer. Entitiles will generate these layers
/// acoording to the LDtk file.
#[derive(Resource, Default, Reflect)]
pub struct LdtkAdditionalLayers {
    #[cfg(feature = "algorithm")]
    pub path_layer: Option<super::layer::path::LdtkPathLayer>,
    #[cfg(feature = "physics")]
    pub physics_layer: Option<super::layer::physics::LdtkPhysicsLayer>,
}

/// Configuration for loading the LDtk file.
#[derive(Resource, Default, Reflect)]
pub struct LdtkLevelConfig {
    pub asset_path_prefix: String,
    #[reflect(ignore)]
    pub filter_mode: FilterMode,
    pub z_index: f32,
    /// Map a certain texture index to a animation.
    pub animation_mapper: HashMap<u32, RawTileAnimation>,
    pub ignore_unregistered_entities: bool,
    pub ignore_unregistered_entity_tags: bool,
}

#[derive(Resource, Default, Deref)]
pub struct LdtkLoadedLevels(pub(crate) HashMap<AssetId<LdtkJson>, HashMap<LevelIid, Entity>>);

impl LdtkLoadedLevels {
    /// Quick utility to unload all levels in all files.
    pub fn unload_all(&self, event_writer: &mut EventWriter<LdtkLevelEvent>) {
        event_writer.send_batch(self.iter().flat_map(|(id, file)| {
            file.keys().map(|iid| {
                LdtkLevelEvent::Unload(LdtkLevelUnloader {
                    json: *id,
                    level: LdtkLevel::Iid(iid.clone()),
                })
            })
        }));
    }

    /// Quick utility to unload all levels in the specific file.
    #[inline]
    pub fn unload_all_at(
        &self,
        json: AssetId<LdtkJson>,
        event_writer: &mut EventWriter<LdtkLevelEvent>,
    ) {
        if let Some(file) = self.get(&json) {
            event_writer.send_batch(file.keys().map(|iid| {
                LdtkLevelEvent::Unload(LdtkLevelUnloader {
                    json,
                    level: LdtkLevel::Iid(iid.clone()),
                })
            }));
        }
    }
}

#[derive(Error, Debug)]
pub enum LdtkJsonLoadError {
    #[error("Io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Json error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Default)]
pub struct LdtkJsonLoader;

impl AssetLoader for LdtkJsonLoader {
    type Asset = LdtkJson;

    type Settings = ();

    type Error = LdtkJsonLoadError;

    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a Self::Settings,
        _load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).await?;
        serde_json::from_slice(&buf).map_err(Into::into)
    }
}

#[derive(Resource, Default, Reflect, Deref, DerefMut)]
pub struct LdtkGlobalEntityRegistry(pub(crate) HashMap<EntityIid, Entity>);
