use std::{fs::read_to_string, path::Path};

use bevy::{
    asset::{AssetServer, Assets, Handle},
    ecs::{
        entity::Entity,
        system::{Commands, Resource},
    },
    log::error,
    math::{IVec2, UVec2, Vec2},
    reflect::Reflect,
    render::{
        mesh::{Indices, Mesh},
        render_resource::{FilterMode, PrimitiveTopology},
    },
    sprite::{Mesh2dHandle, SpriteBundle, TextureAtlas},
    utils::HashMap,
};

use crate::tilemap::tile::RawTileAnimation;
use crate::{
    serializing::pattern::TilemapPattern,
    tilemap::map::{TilemapRotation, TilemapTexture, TilemapTextureDescriptor},
};

use super::{
    components::EntityIid,
    json::{definitions::EntityDef, EntityRef, LdtkJson, TocInstance},
    sprite::{AtlasRect, LdtkEntityMaterial},
    LdtkLoader, LdtkLoaderMode, LdtkUnloader,
};

#[derive(Resource, Reflect, Default, Clone)]
pub struct LdtkPatterns {
    #[reflect(ignore)]
    pub patterns: HashMap<String, (Vec<(TilemapPattern, TilemapTexture)>, SpriteBundle)>,
    #[cfg(feature = "physics")]
    pub physics_patterns: HashMap<String, crate::tilemap::physics::DataPhysicsTilemap>,
    #[cfg(feature = "physics")]
    pub physics_parent: String,
    pub idents: HashMap<u8, String>,
}

impl LdtkPatterns {
    #[inline]
    pub fn new(idents: HashMap<u8, String>) -> Self {
        Self {
            idents,
            ..Default::default()
        }
    }

    #[inline]
    pub fn get_with_ident(
        &self,
        identifier: String,
    ) -> &(Vec<(TilemapPattern, TilemapTexture)>, SpriteBundle) {
        self.patterns.get(&identifier).unwrap()
    }

    #[inline]
    pub fn get_with_index(
        &self,
        index: u8,
    ) -> &(Vec<(TilemapPattern, TilemapTexture)>, SpriteBundle) {
        self.patterns.get(&self.idents[&index]).unwrap()
    }

    #[inline]
    pub fn insert(
        &mut self,
        identifier: String,
        patterns: Vec<(TilemapPattern, TilemapTexture)>,
        background: SpriteBundle,
    ) {
        self.patterns
            .insert(identifier.to_string(), (patterns, background));
    }

    #[inline]
    pub fn get_physics_with_index(
        &self,
        index: u8,
    ) -> Option<&crate::tilemap::physics::DataPhysicsTilemap> {
        self.physics_patterns.get(&self.idents[&index])
    }

    #[inline]
    pub fn is_ready(&self) -> bool {
        !self.patterns.is_empty() && !self.idents.is_empty()
    }
}

#[derive(Resource, Default, Reflect)]
pub struct LdtkAssets {
    pub(crate) associated_file: String,
    /// tileset iid to texture
    pub(crate) tilesets: HashMap<i32, TilemapTexture>,
    /// tileset iid to texture atlas handle
    pub(crate) atlas_handles: HashMap<i32, Handle<TextureAtlas>>,
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

    pub fn clone_atlas_handle(&self, tileset_uid: i32) -> Handle<TextureAtlas> {
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
    pub fn initialize(
        &mut self,
        config: &LdtkLoadConfig,
        manager: &LdtkLevelManager,
        asset_server: &AssetServer,
        atlas_assets: &mut Assets<TextureAtlas>,
        material_assets: &mut Assets<LdtkEntityMaterial>,
        mesh_assets: &mut Assets<Mesh>,
    ) {
        self.associated_file = config.file_path.clone();
        self.load_texture(config, manager, asset_server, atlas_assets);
        self.load_entities(config, manager, material_assets, mesh_assets);
    }

    fn load_texture(
        &mut self,
        config: &LdtkLoadConfig,
        manager: &LdtkLevelManager,
        asset_server: &AssetServer,
        atlas_assets: &mut Assets<TextureAtlas>,
    ) {
        let ldtk_data = manager.get_cached_data();
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
                filter_mode: config.filter_mode,
            };
            let texture = TilemapTexture {
                texture,
                desc,
                rotation: TilemapRotation::None,
            };

            self.tilesets.insert(tileset.uid, texture.clone());
            self.atlas_handles
                .insert(tileset.uid, atlas_assets.add(texture.as_texture_atlas()));
        });
    }

    fn load_entities(
        &mut self,
        config: &LdtkLoadConfig,
        manager: &LdtkLevelManager,
        material_assets: &mut Assets<LdtkEntityMaterial>,
        mesh_assets: &mut Assets<Mesh>,
    ) {
        let ldtk_data = manager.get_cached_data();
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

                let mesh = Mesh::new(PrimitiveTopology::TriangleList)
                    .with_inserted_attribute(
                        Mesh::ATTRIBUTE_POSITION,
                        sprite_mesh
                            .vertices
                            .into_iter()
                            .map(|p| p.extend(entity_depth[&entity_instance.identifier]))
                            .collect::<Vec<_>>(),
                    )
                    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, sprite_mesh.uvs)
                    .with_indices(Some(Indices::U16(sprite_mesh.indices)));
                self.meshes
                    .insert(entity_instance.iid.clone(), mesh_assets.add(mesh).into());
            });
    }
}

#[cfg(feature = "algorithm")]
#[derive(Resource, Default, Reflect)]
pub struct LdtkWfcManager {
    pub(crate) wfc_data: Option<crate::algorithm::wfc::WfcData>,
    pub(crate) idents: HashMap<u8, String>,
    pub(crate) pattern_size: Vec2,
}

#[cfg(feature = "algorithm")]
impl LdtkWfcManager {
    pub fn get_ident(&self, level_index: UVec2) -> Option<String> {
        let idx = self.wfc_data.as_ref()?.get(level_index)?;
        Some(self.idents[&idx].clone())
    }

    pub fn get_translation(&self, level_index: IVec2) -> Vec2 {
        (level_index.as_vec2() + Vec2::Y) * self.pattern_size
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

#[derive(Resource, Default, Reflect)]
pub struct LdtkAdditionalLayers {
    #[cfg(feature = "algorithm")]
    pub path_layer: Option<super::layer::path::LdtkPathLayer>,
    #[cfg(feature = "physics")]
    pub physics_layer: Option<super::layer::physics::LdtkPhysicsLayer>,
}

#[derive(Resource, Default, Reflect)]
pub struct LdtkLoadConfig {
    pub file_path: String,
    pub asset_path_prefix: String,
    #[reflect(ignore)]
    pub filter_mode: FilterMode,
    pub z_index: i32,
    /// Map a certain texture index to a animation.
    pub animation_mapper: HashMap<u32, RawTileAnimation>,
    pub ignore_unregistered_entities: bool,
    pub ignore_unregistered_entity_tags: bool,
}

#[derive(Resource, Default, Reflect)]
pub struct LdtkLevelManager {
    pub(crate) ldtk_json: Option<LdtkJson>,
    pub(crate) loaded_levels: HashMap<String, Entity>,
}

impl LdtkLevelManager {
    /// Reloads the ldtk file and refresh the level cache.
    pub fn reload_json(&mut self, config: &LdtkLoadConfig) {
        if config.file_path.is_empty() {
            error!("No specified ldtk level file path!");
            return;
        }

        let path = std::env::current_dir().unwrap().join(&config.file_path);
        let str_raw = match read_to_string(&path) {
            Ok(data) => data,
            Err(e) => panic!("Could not read file at path: {:?}!\n{}", path, e),
        };

        self.ldtk_json = match serde_json::from_str::<LdtkJson>(&str_raw) {
            Ok(data) => Some(data),
            Err(e) => panic!("Could not parse file at path: {}!\n{}", config.file_path, e),
        };
    }

    pub fn get_cached_data(&self) -> &LdtkJson {
        self.check_initialized();
        self.ldtk_json.as_ref().unwrap()
    }

    pub fn load(&mut self, commands: &mut Commands, level: String, trans_ovrd: Option<Vec2>) {
        let level = level.to_string();

        if self.loaded_levels.contains_key(&level.to_string()) {
            error!("Trying to load {:?} that is already loaded!", level);
        } else {
            let entity = commands.spawn(LdtkLoader {
                level: level.clone(),
                mode: LdtkLoaderMode::Tilemap,
                trans_ovrd,
            });
            self.loaded_levels.insert(level.clone(), entity.id());
        }
    }

    pub fn load_all_patterns(&mut self, commands: &mut Commands) {
        self.check_initialized();

        self.ldtk_json
            .as_ref()
            .unwrap()
            .levels
            .iter()
            .for_each(|level| {
                if self.loaded_levels.contains_key(&level.identifier) {
                    error!("Trying to load {:?} that is already loaded!", level);
                } else {
                    commands.spawn(LdtkLoader {
                        level: level.identifier.clone(),
                        mode: LdtkLoaderMode::MapPattern,
                        trans_ovrd: None,
                    });
                }
            });
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
            commands.entity(*l).insert(LdtkUnloader);
            self.loaded_levels.remove(&level);
        } else {
            error!("Trying to unload {:?} that is not loaded!", level);
        }
    }

    pub fn unload_all(&mut self, commands: &mut Commands) {
        for (_, l) in self.loaded_levels.iter() {
            commands.entity(*l).insert(LdtkUnloader);
        }
        self.loaded_levels.clear();
    }

    pub fn is_loaded(&self, level: String) -> bool {
        self.loaded_levels.contains_key(&level)
    }

    pub fn is_initialized(&self) -> bool {
        self.ldtk_json.is_some()
    }

    fn check_initialized(&self) {
        assert!(
            self.is_initialized(),
            "LdtkLevelManager is not initialized!"
        );
    }
}

#[derive(Resource, Default, Reflect)]
pub struct LdtkGlobalEntityRegistry(pub(crate) HashMap<EntityIid, Entity>);

impl LdtkGlobalEntityRegistry {
    #[inline]
    pub fn register(&mut self, iid: EntityIid, entity: Entity) {
        self.0.insert(iid, entity);
    }

    #[inline]
    pub fn contains(&self, iid: &EntityIid) -> bool {
        self.0.contains_key(iid)
    }

    #[inline]
    pub fn get(&self, iid: &EntityIid) -> Option<Entity> {
        self.0.get(iid).cloned()
    }

    #[inline]
    pub fn remove(&mut self, iid: &EntityIid) -> Option<Entity> {
        self.0.remove(iid)
    }

    #[inline]
    pub fn remove_all(&mut self) {
        self.0.clear();
    }

    #[inline]
    pub fn despawn(&mut self, commands: &mut Commands, iid: &EntityIid) {
        if let Some(entity) = self.remove(iid) {
            commands.entity(entity).despawn();
        }
    }

    #[inline]
    pub fn despawn_all(&mut self, commands: &mut Commands) {
        self.0.iter().for_each(|(_, entity)| {
            commands.entity(*entity).despawn();
        });
        self.remove_all();
    }
}
