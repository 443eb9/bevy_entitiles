use bevy::{
    ecs::{
        entity::Entity,
        system::{Commands, Resource},
    },
    hierarchy::DespawnRecursiveExt,
    log::error,
    math::Vec2,
    render::render_resource::FilterMode,
    utils::HashMap,
};

use super::{LdtkLevelIdent, LdtkLoader};

#[derive(Resource, Default)]
pub struct LdtkLevelManager {
    file_path: String,
    asset_path_prefix: String,
    level_spacing: Option<i32>,
    filter_mode: FilterMode,
    ignore_unregistered_entities: bool,
    z_index: i32,
    atlas_render_size: HashMap<String, Vec2>,

    loaded_levels: HashMap<LdtkLevelIdent, Entity>,
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

    /// The render size for tile atlas.
    pub fn set_atlas_render_size(&mut self, identifier: String, size: Vec2) -> &mut Self {
        self.atlas_render_size.insert(identifier, size);
        self
    }

    pub fn load(&mut self, commands: &mut Commands, level: LdtkLevelIdent) {
        self.check_initialized();
        if self.loaded_levels.contains_key(&level) {
            error!("Trying to load {:?} that is already loaded!", level);
        } else {
            let mut loader = self.generate_loader();
            loader.level = level.clone();
            self.loaded_levels
                .insert(level, commands.spawn(loader).id());
        }
    }

    pub fn unload(&mut self, commands: &mut Commands, level: LdtkLevelIdent) {
        if let Some(l) = self.loaded_levels.get(&level) {
            commands.entity(*l).despawn_recursive();
            self.loaded_levels.remove(&level);
        } else {
            error!("Trying to unload {:?} that is not loaded!", level);
        }
    }

    pub fn is_loaded(&self, level: LdtkLevelIdent) -> bool {
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
            level: LdtkLevelIdent::None,
            level_spacing: self.level_spacing,
            ignore_unregistered_entities: self.ignore_unregistered_entities,
            z_index: self.z_index,
            atlas_render_size: self.atlas_render_size.clone(),
        }
    }
}
