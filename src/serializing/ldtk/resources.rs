use std::fs::read_to_string;

use bevy::{
    asset::Handle,
    ecs::{
        entity::Entity,
        system::{Commands, Resource},
    },
    log::error,
    render::render_resource::FilterMode,
    sprite::TextureAtlas,
    utils::HashMap,
};

use crate::render::texture::TilemapTexture;

use super::{json::LdtkJson, physics::LdtkPhysicsLayer, LdtkLoader, LdtkUnloader};

#[derive(Resource, Default)]
pub struct LdtkLevelManager {
    pub(crate) file_path: String,
    pub(crate) asset_path_prefix: String,
    pub(crate) ldtk_json: Option<LdtkJson>,
    pub(crate) level_spacing: Option<i32>,
    pub(crate) filter_mode: FilterMode,
    pub(crate) ignore_unregistered_entities: bool,
    pub(crate) z_index: i32,
    pub(crate) loaded_levels: HashMap<String, Entity>,
    pub(crate) physics_layer: Option<LdtkPhysicsLayer>,
}

impl LdtkLevelManager {
    /// `file_path`: The path to the ldtk file relative to the working directory.
    ///
    /// `asset_path_prefix`: The path to the ldtk file relative to the assets folder.
    ///
    /// For example, your ldtk file is located at `assets/ldtk/fantastic_map.ldtk`,
    /// so `asset_path_prefix` will be `ldtk/`.
    pub fn initialize(&mut self, file_path: String, asset_path_prefix: String) -> &mut Self {
        self.file_path = file_path;
        self.asset_path_prefix = asset_path_prefix;
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

    /// The identifier of the physics layer.
    /// Set this to allow the algorithm to figure out the colliders.
    /// The layer you specify must be an int grid, or the program will panic.
    ///
    /// The `air_value` is the value of the tiles in the int grid which will be considered as air.
    pub fn set_physics_layer(&mut self, physics: LdtkPhysicsLayer) -> &mut Self {
        self.physics_layer = Some(physics);
        self
    }

    /// The filter mode of the tilemap texture.
    pub fn set_filter_mode(&mut self, filter_mode: FilterMode) -> &mut Self {
        self.filter_mode = filter_mode;
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
        if !self.loaded_levels.is_empty() {
            panic!(
                "It's not allowed to load a level when there are already loaded levels! \
                See Known Issues in README.md to know why."
            )
        }

        if self.loaded_levels.contains_key(&level.to_string()) {
            error!("Trying to load {:?} that is already loaded!", level);
        } else {
            let mut loader = self.generate_loader();
            loader.level = level.clone();
            self.loaded_levels
                .insert(level, commands.spawn(loader).id());
        }
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
            let level = level.to_string();
            if self.loaded_levels.contains_key(&level.to_string()) {
                error!("Trying to load {:?} that is already loaded!", level);
            } else {
                let mut loader = self.generate_loader();
                loader.level = level.clone();
                self.loaded_levels
                    .insert(level, commands.spawn(loader).id());
            }
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
        if !self.is_initialized() {
            panic!("LdtkLevelManager is not initialized!");
        }
    }

    fn generate_loader(&self) -> LdtkLoader {
        LdtkLoader {
            level: "".to_string(),
        }
    }
}

#[derive(Resource, Default)]
pub struct LdtkTextures {
    pub(crate) tilesets: HashMap<i32, TilemapTexture>,
    pub(crate) atlas_handles: HashMap<i32, Handle<TextureAtlas>>,
}

impl LdtkTextures {
    pub fn insert_tileset(&mut self, id: i32, tileset: TilemapTexture) {
        self.tilesets.insert(id, tileset);
    }

    pub fn insert_atlas(&mut self, id: i32, atlas: Handle<TextureAtlas>) {
        self.atlas_handles.insert(id, atlas);
    }

    pub fn get_tileset(&self, id: i32) -> Option<&TilemapTexture> {
        self.tilesets.get(&id)
    }

    pub fn get_atlas(&self, id: i32) -> Option<&Handle<TextureAtlas>> {
        self.atlas_handles.get(&id)
    }
}
