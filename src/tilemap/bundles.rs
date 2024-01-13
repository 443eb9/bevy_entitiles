use bevy::{
    ecs::bundle::Bundle,
    render::view::{InheritedVisibility, ViewVisibility, Visibility},
    transform::components::{GlobalTransform, Transform},
};

use super::map::{
    TilePivot, TileRenderSize, TilemapAabbs, TilemapAnimations, TilemapLayerOpacities, TilemapName,
    TilemapSlotSize, TilemapStorage, TilemapTexture, TilemapTransform, TilemapType,
};

#[derive(Bundle, Default, Debug, Clone)]
pub struct TilemapBundle {
    pub name: TilemapName,
    pub tile_render_size: TileRenderSize,
    pub slot_size: TilemapSlotSize,
    pub ty: TilemapType,
    pub tile_pivot: TilePivot,
    pub layer_opacities: TilemapLayerOpacities,
    pub storage: TilemapStorage,
    pub tilemap_transform: TilemapTransform,
    pub texture: TilemapTexture,
    pub animations: TilemapAnimations,
    pub aabbs: TilemapAabbs,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

#[derive(Bundle, Default, Debug, Clone)]
pub struct PureColorTilemapBundle {
    pub name: TilemapName,
    pub tile_render_size: TileRenderSize,
    pub slot_size: TilemapSlotSize,
    pub ty: TilemapType,
    pub tile_pivot: TilePivot,
    pub layer_opacities: TilemapLayerOpacities,
    pub storage: TilemapStorage,
    pub tilemap_transform: TilemapTransform,
    pub aabbs: TilemapAabbs,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}
