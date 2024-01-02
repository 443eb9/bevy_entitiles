use std::fs::read_to_string;

use bevy::{
    asset::{AssetServer, Assets, Handle},
    ecs::{
        entity::Entity,
        system::{Commands, Resource, SystemId},
    },
    log::error,
    math::{IVec2, UVec2, Vec4},
    reflect::Reflect,
    render::{
        mesh::{Indices, Mesh},
        render_resource::{FilterMode, PrimitiveTopology},
        texture::Image,
    },
    sprite::{Mesh2dHandle, TextureAtlas},
    utils::HashMap,
};

use crate::{
    reflect::ReflectFilterMode,
    render::texture::{TilemapTexture, TilemapTextureDescriptor},
    serializing::pattern::TilemapPattern,
    tilemap::map::TilemapRotation,
};

use super::{
    json::{definitions::EntityDef, LdtkJson},
    sprite::{AtlasRect, LdtkEntityMaterial},
    LdtkLoader, LdtkUnloader,
};

#[derive(Reflect, Clone)]
pub enum LdtkBackground {
    Color(Vec4),
    Texture(Handle<Image>),
}

#[derive(Resource, Reflect, Default, Clone)]
pub struct LdtkPatterns {
    pub patterns: HashMap<String, (Vec<TilemapPattern>, TilemapTexture)>,
    #[reflect(ignore)]
    pub callback: Option<SystemId>,
    pub threashold: Option<usize>,
}

impl LdtkPatterns {
    pub fn get(&self, identifier: String) -> &(Vec<TilemapPattern>, TilemapTexture) {
        self.patterns.get(&identifier).unwrap()
    }

    pub fn insert(
        &mut self,
        commands: &mut Commands,
        identifier: String,
        patterns: Vec<TilemapPattern>,
        texture: TilemapTexture,
    ) {
        self.patterns
            .insert(identifier.to_string(), (patterns, texture));

        if let (Some(threashold), Some(callback)) = (self.threashold, self.callback) {
            if self.patterns.len() == threashold {
                commands.run_system(callback);
            }
        }
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
        manager: &LdtkLevelManager,
        asset_server: &AssetServer,
        atlas_assets: &mut Assets<TextureAtlas>,
        material_assets: &mut Assets<LdtkEntityMaterial>,
        mesh_assets: &mut Assets<Mesh>,
    ) {
        self.associated_file = manager.file_path.clone();
        self.load_texture(manager, asset_server, atlas_assets);
        self.load_entities(manager, material_assets, mesh_assets);
    }

    fn load_texture(
        &mut self,
        manager: &LdtkLevelManager,
        asset_server: &AssetServer,
        atlas_assets: &mut Assets<TextureAtlas>,
    ) {
        let ldtk_data = manager.get_cached_data();
        ldtk_data.defs.tilesets.iter().for_each(|tileset| {
            let Some(path) = tileset.rel_path.as_ref() else {
                return;
            };

            let texture = asset_server.load(format!("{}{}", manager.asset_path_prefix, path));
            let desc = TilemapTextureDescriptor {
                size: UVec2 {
                    x: tileset.px_wid as u32,
                    y: tileset.px_hei as u32,
                },
                tile_size: UVec2 {
                    x: tileset.tile_grid_size as u32,
                    y: tileset.tile_grid_size as u32,
                },
                filter_mode: manager.filter_mode.into(),
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
                            (ldtk_data.defs.entities.len() - index) as f32 + manager.z_index as f32,
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

#[derive(Reflect, Default, Clone, Copy, PartialEq, Eq)]
pub enum LdtkLevelManagerMode {
    #[default]
    Tilemap,
    MapPattern,
}

#[derive(Resource, Default, Reflect)]
pub struct LdtkLevelManager {
    pub(crate) file_path: String,
    pub(crate) asset_path_prefix: String,
    pub(crate) mode: LdtkLevelManagerMode,
    pub(crate) ldtk_json: Option<LdtkJson>,
    pub(crate) level_spacing: Option<i32>,
    pub(crate) filter_mode: ReflectFilterMode,
    pub(crate) ignore_unregistered_entities: bool,
    pub(crate) z_index: i32,
    pub(crate) loaded_levels: HashMap<String, Entity>,
    pub(crate) global_entities: HashMap<String, Entity>,
    #[cfg(feature = "algorithm")]
    pub(crate) path_layer: Option<super::layer::path::LdtkPathLayer>,
    #[cfg(any(feature = "physics_xpbd", feature = "physics_rapier"))]
    pub(crate) physics_layer: Option<super::layer::physics::LdtkPhysicsLayer>,
}

impl LdtkLevelManager {
    pub fn new(file_path: &str, asset_path_prefix: &str, mode: LdtkLevelManagerMode) -> Self {
        let mut s = Self {
            file_path: file_path.to_string(),
            asset_path_prefix: asset_path_prefix.to_string(),
            ..Default::default()
        };
        s.initialize(file_path.to_string(), asset_path_prefix.to_string(), mode);
        s
    }

    /// `file_path`: The path to the ldtk file relative to the working directory.
    ///
    /// `asset_path_prefix`: The path to the ldtk file relative to the assets folder.
    ///
    /// For example, your ldtk file is located at `assets/ldtk/fantastic_map.ldtk`,
    /// so `asset_path_prefix` will be `ldtk/`.
    pub fn initialize(
        &mut self,
        file_path: String,
        asset_path_prefix: String,
        mode: LdtkLevelManagerMode,
    ) -> &mut Self {
        self.file_path = file_path;
        self.asset_path_prefix = asset_path_prefix;
        self.mode = mode;
        self.reload_json();
        self
    }

    /// Reloads the ldtk file and refresh the level cache.
    pub fn reload_json(&mut self) {
        let path = std::env::current_dir().unwrap().join(&self.file_path);
        let str_raw = match read_to_string(&path) {
            Ok(data) => data,
            Err(e) => panic!("Could not read file at path: {:?}!\n{}", path, e),
        };

        self.ldtk_json = match serde_json::from_str::<LdtkJson>(&str_raw) {
            Ok(data) => Some(data),
            Err(e) => panic!("Could not parse file at path: {}!\n{}", self.file_path, e),
        };
    }

    /// If you are using a map with `WorldLayout::LinearHorizontal` or `WorldLayout::LinearVertical` layout,
    /// and you are going to load all the levels,
    /// this value will be used to determine the spacing between the levels.
    pub fn set_level_spacing(&mut self, level_spacing: i32) -> &mut Self {
        self.level_spacing = Some(level_spacing);
        self
    }

    /// Set this to allow the algorithm to figure out the colliders.
    /// The layer you specify must be an int grid, or the program will panic.
    ///
    /// The `air_value` is the value of the tiles in the int grid which will be considered as air.
    #[cfg(any(feature = "physics_xpbd", feature = "physics_rapier"))]
    pub fn set_physics_layer(
        &mut self,
        physics: super::layer::physics::LdtkPhysicsLayer,
    ) -> &mut Self {
        self.physics_layer = Some(physics);
        self
    }

    /// Set this to allow automatic path tilemap generating.
    #[cfg(feature = "algorithm")]
    pub fn set_path_layer(&mut self, path: super::layer::path::LdtkPathLayer) -> &mut Self {
        self.path_layer = Some(path);
        self
    }

    /// The filter mode of the tilemap texture.
    pub fn set_filter_mode(&mut self, filter_mode: FilterMode) -> &mut Self {
        self.filter_mode = filter_mode.into();
        self
    }

    /// If `true`, then the entities with unregistered identifiers will be ignored.
    /// If `false`, then the program will panic.
    pub fn set_if_ignore_unregistered_entities(&mut self, is_ignore: bool) -> &mut Self {
        self.ignore_unregistered_entities = is_ignore;
        self
    }

    /// The z index of the tilemap will be `base_z_index - level_index`.
    pub fn set_base_z_index(&mut self, z_index: i32) -> &mut Self {
        self.z_index = z_index;
        self
    }

    pub fn get_cached_data(&self) -> &LdtkJson {
        self.check_initialized();
        self.ldtk_json.as_ref().unwrap()
    }

    pub fn load(&mut self, commands: &mut Commands, level: &'static str) {
        self.check_initialized();
        let level = level.to_string();
        assert!(
            self.loaded_levels.is_empty(),
            "It's not allowed to load a level when there are already loaded levels! \
            See Known Issues in README.md to know why."
        );

        if self.loaded_levels.contains_key(&level.to_string()) {
            error!("Trying to load {:?} that is already loaded!", level);
        } else {
            let entity = commands.spawn(LdtkLoader {
                level: level.clone(),
            });
            match self.mode {
                LdtkLevelManagerMode::Tilemap => {
                    self.loaded_levels.insert(level.clone(), entity.id());
                }
                LdtkLevelManagerMode::MapPattern => {}
            }
        }
    }

    pub fn load_all(&mut self, commands: &mut Commands) {
        self.check_initialized();
        assert!(
            self.loaded_levels.is_empty(),
            "It's not allowed to load a level when there are already loaded levels! \
            See Known Issues in README.md to know why."
        );

        self.ldtk_json
            .as_ref()
            .unwrap()
            .levels
            .iter()
            .for_each(|level| {
                if self.loaded_levels.contains_key(&level.identifier) {
                    error!("Trying to load {:?} that is already loaded!", level);
                } else {
                    let entity = commands.spawn(LdtkLoader {
                        level: level.identifier.clone(),
                    });
                    match self.mode {
                        LdtkLevelManagerMode::Tilemap => {
                            self.loaded_levels
                                .insert(level.identifier.clone(), entity.id());
                        }
                        LdtkLevelManagerMode::MapPattern => {}
                    }
                }
            });
    }

    pub fn try_load(&mut self, commands: &mut Commands, level: &'static str) -> bool {
        self.check_initialized();
        if self.loaded_levels.is_empty() {
            self.load(commands, level);
            true
        } else {
            false
        }
    }

    pub fn switch_to(&mut self, commands: &mut Commands, level: &'static str) {
        self.check_initialized();
        if self.loaded_levels.contains_key(&level.to_string()) {
            error!("Trying to load {:?} that is already loaded!", level);
        } else {
            self.unload_all(commands);
            self.load(commands, level);
        }
    }

    /// # Warning!
    ///
    /// This method will cause panic if you have already loaded levels before.
    /// **Even if you have unloaded them!!**
    pub fn load_many(&mut self, commands: &mut Commands, levels: &[&'static str]) {
        self.check_initialized();
        levels.iter().for_each(|level| {
            self.load(commands, &level);
        });
    }

    /// # Warning!
    ///
    /// This method will cause panic if you have already loaded levels before.
    /// **Even if you have unloaded them!!**
    pub fn try_load_many(&mut self, commands: &mut Commands, levels: &[&'static str]) -> bool {
        self.check_initialized();
        if self.loaded_levels.is_empty() {
            self.load_many(commands, levels);
            true
        } else {
            false
        }
    }

    pub fn unload(&mut self, commands: &mut Commands, level: &'static str) {
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
