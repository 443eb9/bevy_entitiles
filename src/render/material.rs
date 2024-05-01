use bevy::{
    asset::{Asset, AssetId},
    ecs::system::Resource,
    reflect::TypePath,
    render::color::Color,
    utils::HashMap,
};

use crate::tilemap::map::TilemapTexture;

#[derive(Default, Asset, TypePath, Clone)]
pub struct StandardTilemapMaterial {
    pub tint: Color,
    pub texture: Option<TilemapTexture>,
}

#[derive(Resource, Default)]
pub struct StandardTilemapMaterialInstances {
    instances: HashMap<AssetId<StandardTilemapMaterial>, StandardTilemapMaterial>,
}

impl StandardTilemapMaterialInstances {
    #[inline]
    pub fn get(
        &self,
        handle: &AssetId<StandardTilemapMaterial>,
    ) -> Option<&StandardTilemapMaterial> {
        self.instances.get(handle)
    }

    #[inline]
    pub fn remove(&mut self, handle: &AssetId<StandardTilemapMaterial>) {
        self.instances.remove(handle);
    }

    #[inline]
    pub fn insert(
        &mut self,
        handle: AssetId<StandardTilemapMaterial>,
        material: StandardTilemapMaterial,
    ) {
        self.instances.insert(handle, material);
    }
}

#[derive(Resource, Default)]
pub struct ExtractedStandardTilemapMaterials {
    pub extracted: Vec<(AssetId<StandardTilemapMaterial>, StandardTilemapMaterial)>,
    pub removed: Vec<AssetId<StandardTilemapMaterial>>,
}

#[derive(Default)]
pub struct PrepareNextFrameStdTilemapMaterials {
    pub assets: Vec<(AssetId<StandardTilemapMaterial>, StandardTilemapMaterial)>,
}
