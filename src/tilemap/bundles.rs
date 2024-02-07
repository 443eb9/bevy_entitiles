use bevy::{asset::Handle, ecs::bundle::Bundle};

use crate::render::material::{
    StandardTilemapMaterial, TilemapMaterial, WaitForStandardMaterialRepacement,
};

use super::map::{
    TilePivot, TileRenderSize, TilemapAnimations, TilemapAxisFlip, TilemapLayerOpacities,
    TilemapName, TilemapSlotSize, TilemapStorage, TilemapTexture, TilemapTransform, TilemapType,
    WaitForTextureUsageChange,
};

/// All the possible bundles of the tilemap.
#[derive(Debug, Clone)]
pub enum TilemapBundles {
    Data(DataTilemapBundle),
    PureColor(StandardPureColorTilemapBundle),
    Texture(StandardTilemapBundle),
}

/// The bundle of the tilemap with no actual tiles.
#[derive(Bundle, Default, Debug, Clone)]
pub struct DataTilemapBundle {
    pub name: TilemapName,
    pub tile_render_size: TileRenderSize,
    pub slot_size: TilemapSlotSize,
    pub ty: TilemapType,
    pub tile_pivot: TilePivot,
    pub axis_direction: TilemapAxisFlip,
}

impl Into<StandardTilemapBundle> for DataTilemapBundle {
    fn into(self) -> StandardTilemapBundle {
        StandardTilemapBundle {
            name: self.name,
            tile_render_size: self.tile_render_size,
            slot_size: self.slot_size,
            ty: self.ty,
            tile_pivot: self.tile_pivot,
            ..Default::default()
        }
    }
}

/// The bundle of the tilemap with a texture and custom material.
#[derive(Bundle, Default, Debug, Clone)]
pub struct MaterialTilemapBundle<M: TilemapMaterial> {
    pub name: TilemapName,
    pub tile_render_size: TileRenderSize,
    pub slot_size: TilemapSlotSize,
    pub ty: TilemapType,
    pub tile_pivot: TilePivot,
    pub layer_opacities: TilemapLayerOpacities,
    pub storage: TilemapStorage,
    pub transform: TilemapTransform,
    pub axis_flip: TilemapAxisFlip,
    pub material: Handle<M>,
    pub texture: TilemapTexture,
    pub animations: TilemapAnimations,
    pub marker: WaitForTextureUsageChange,
}

/// The bundle of the tilemap with a texture and standard material.
#[derive(Bundle, Default, Debug, Clone)]
pub struct StandardTilemapBundle {
    pub name: TilemapName,
    pub tile_render_size: TileRenderSize,
    pub slot_size: TilemapSlotSize,
    pub ty: TilemapType,
    pub tile_pivot: TilePivot,
    pub layer_opacities: TilemapLayerOpacities,
    pub storage: TilemapStorage,
    pub transform: TilemapTransform,
    pub axis_flip: TilemapAxisFlip,
    pub material: Handle<StandardTilemapMaterial>,
    pub texture: TilemapTexture,
    pub animations: TilemapAnimations,
    pub texture_marker: WaitForTextureUsageChange,
    pub material_marker: WaitForStandardMaterialRepacement,
}

impl Into<DataTilemapBundle> for StandardTilemapBundle {
    fn into(self) -> DataTilemapBundle {
        DataTilemapBundle {
            name: self.name,
            tile_render_size: self.tile_render_size,
            slot_size: self.slot_size,
            ty: self.ty,
            tile_pivot: self.tile_pivot,
            ..Default::default()
        }
    }
}

impl Into<StandardPureColorTilemapBundle> for StandardTilemapBundle {
    fn into(self) -> StandardPureColorTilemapBundle {
        StandardPureColorTilemapBundle {
            name: self.name,
            tile_render_size: self.tile_render_size,
            slot_size: self.slot_size,
            ty: self.ty,
            tile_pivot: self.tile_pivot,
            layer_opacities: self.layer_opacities,
            storage: self.storage,
            transform: self.transform,
            axis_flip: self.axis_flip,
            material: self.material,
            material_marker: self.material_marker,
        }
    }
}

/// The bundle of the tilemap without a texture and with a custom material.
/// This can be cheaper.
#[derive(Bundle, Default, Debug, Clone)]
pub struct PureColorTilemapBundle<M: TilemapMaterial> {
    pub name: TilemapName,
    pub tile_render_size: TileRenderSize,
    pub slot_size: TilemapSlotSize,
    pub ty: TilemapType,
    pub tile_pivot: TilePivot,
    pub layer_opacities: TilemapLayerOpacities,
    pub storage: TilemapStorage,
    pub transform: TilemapTransform,
    pub axis_flip: TilemapAxisFlip,
    pub material: Handle<M>,
}

/// The bundle of the tilemap without a texture and with a standard material.
/// This can be cheaper.
#[derive(Bundle, Default, Debug, Clone)]
pub struct StandardPureColorTilemapBundle {
    pub name: TilemapName,
    pub tile_render_size: TileRenderSize,
    pub slot_size: TilemapSlotSize,
    pub ty: TilemapType,
    pub tile_pivot: TilePivot,
    pub layer_opacities: TilemapLayerOpacities,
    pub storage: TilemapStorage,
    pub transform: TilemapTransform,
    pub axis_flip: TilemapAxisFlip,
    pub material: Handle<StandardTilemapMaterial>,
    pub material_marker: WaitForStandardMaterialRepacement,
}

impl StandardPureColorTilemapBundle {
    pub fn convert_to_texture_bundle(
        self,
        texture: TilemapTexture,
        animations: TilemapAnimations,
    ) -> StandardTilemapBundle {
        StandardTilemapBundle {
            name: self.name,
            tile_render_size: self.tile_render_size,
            slot_size: self.slot_size,
            ty: self.ty,
            tile_pivot: self.tile_pivot,
            layer_opacities: self.layer_opacities,
            storage: self.storage,
            transform: self.transform,
            axis_flip: self.axis_flip,
            material: self.material,
            texture,
            animations,
            texture_marker: Default::default(),
            material_marker: self.material_marker,
        }
    }
}
