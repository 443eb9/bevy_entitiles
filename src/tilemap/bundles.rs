use bevy::{
    asset::Handle,
    ecs::bundle::Bundle,
    render::view::{InheritedVisibility, ViewVisibility, Visibility},
};

use crate::render::material::StandardTilemapMaterial;

use super::map::{
    TilePivot, TileRenderSize, TilemapAabbs, TilemapAnimations, TilemapAxisFlip,
    TilemapLayerOpacities, TilemapName, TilemapSlotSize, TilemapStorage, TilemapTransform,
    TilemapType, WaitForTextureUsageChange,
};

/// All the possible bundles of the tilemap.
#[derive(Debug, Clone)]
pub enum TilemapBundles {
    Data(DataTilemapBundle),
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

// /// The bundle of the tilemap with a texture and custom material.
// #[derive(Bundle, Default, Debug, Clone)]
// pub struct MaterialTilemapBundle<M: TilemapMaterial> {
//     pub name: TilemapName,
//     pub tile_render_size: TileRenderSize,
//     pub slot_size: TilemapSlotSize,
//     pub ty: TilemapType,
//     pub tile_pivot: TilePivot,
//     pub layer_opacities: TilemapLayerOpacities,
//     pub storage: TilemapStorage,
//     pub transform: TilemapTransform,
//     pub axis_flip: TilemapAxisFlip,
//     pub material: Handle<M>,
//     pub animations: TilemapAnimations,
//     pub visibility: Visibility,
//     pub inherited_visibility: InheritedVisibility,
//     pub view_visibility: ViewVisibility,
//     pub aabbs: TilemapAabbs,
//     pub marker: WaitForTextureUsageChange,
// }

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
    pub animations: TilemapAnimations,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
    pub aabbs: TilemapAabbs,
    pub texture_marker: WaitForTextureUsageChange,
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
