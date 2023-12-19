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

use super::{LdtkLoader, LdtkUnloader};

#[derive(Resource, Default)]
pub struct LdtkLevelManager {
    file_path: String,
    asset_path_prefix: String,
    level_spacing: Option<i32>,
    filter_mode: FilterMode,
    ignore_unregistered_entities: bool,
    z_index: i32,

    loaded_levels: HashMap<String, Entity>,
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
        self
    }

    /// If you are using a map with `WorldLayout::LinearHorizontal` or `WorldLayout::LinearVertical` layout,
    /// and you are going to load all the levels,
    /// this value will be used to determine the spacing between the levels.
    pub fn set_level_spacing(&mut self, level_spacing: i32) -> &mut Self {
        self.level_spacing = Some(level_spacing);
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

    pub fn switch(&mut self, commands: &mut Commands, level: &'static str) {
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

    fn check_initialized(&self) {
        if self.file_path.is_empty() {
            panic!("LdtkLevelManager is not initialized!");
        }
    }

    fn generate_loader(&self) -> LdtkLoader {
        LdtkLoader {
            path: self.file_path.clone(),
            asset_path_prefix: self.asset_path_prefix.clone(),
            filter_mode: self.filter_mode,
            level: "".to_string(),
            level_spacing: self.level_spacing,
            ignore_unregistered_entities: self.ignore_unregistered_entities,
            z_index: self.z_index,
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
