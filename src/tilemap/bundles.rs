use bevy::{
    ecs::bundle::Bundle,
    render::view::{InheritedVisibility, ViewVisibility, Visibility},
    transform::components::{GlobalTransform, Transform},
};

use super::map::{
    TilePivot, TileRenderSize, TilemapAabbs, TilemapAnimations, TilemapLayerOpacities, TilemapName,
    TilemapSlotSize, TilemapStorage, TilemapTexture, TilemapTransform, TilemapType,
};

/// The bundle of the tilemap with a texture.
#[derive(Bundle, Default, Debug, Clone)]
pub struct TilemapBundle {
    /// The name of the tilemap. This can be used in saving the tilemap.
    pub name: TilemapName,
    /// The render size of tiles in pixels.
    pub tile_render_size: TileRenderSize,
    /// The size of each slot in pixels. This can be different from the render size.
    /// And you can create margins and paddings.
    pub slot_size: TilemapSlotSize,
    /// The type of the tilemap.
    pub ty: TilemapType,
    /// The pivot of the tiles. 
    pub tile_pivot: TilePivot,
    /// The opacities of each **rendered** layer.
    /// 
    /// Only the top 4 layers will be rendered.
    pub layer_opacities: TilemapLayerOpacities,
    /// The storage of the tilemap. The entities of each tiles are divided into chunks and stored in it.
    /// 
    /// You need to spawn an empty tilemap and assign it to the storage.
    pub storage: TilemapStorage,
    /// The transform of the tilemap. It's not the same one as `Transform`.
    /// If you want to move or rotate the tilemap, you need to change this.
    /// Modify the `Transform` component will not work.
    pub tilemap_transform: TilemapTransform,
    /// The texture of the tilemap.
    pub texture: TilemapTexture,
    /// All the animation sequences of the tilemap.
    pub animations: TilemapAnimations,
    /// The aabbs of the tilemap. Including the chunk aabbs and the tilemap aabb.
    pub aabbs: TilemapAabbs,
    /// Just to make sure the child sprites are correctly rendered.
    pub visibility: Visibility,
    /// Just to make sure the child sprites are correctly rendered.
    pub inherited_visibility: InheritedVisibility,
    /// Just to make sure the child sprites are correctly rendered.
    pub view_visibility: ViewVisibility,
    /// Modify `TilemapTransform` instead of this one.
    pub transform: Transform,
    /// Just to make sure the child sprites are correctly rendered.
    pub global_transform: GlobalTransform,
}

/// The bundle of the tilemap without a texture. This can be cheaper.
#[derive(Bundle, Default, Debug, Clone)]
pub struct PureColorTilemapBundle {
    /// The name of the tilemap. This can be used in saving the tilemap.
    pub name: TilemapName,
    /// The render size of tiles in pixels.
    pub tile_render_size: TileRenderSize,
    /// The size of each slot in pixels. This can be different from the render size.
    /// And you can create margins and paddings.
    pub slot_size: TilemapSlotSize,
    /// The type of the tilemap.
    pub ty: TilemapType,
    /// The pivot of the tiles. 
    pub tile_pivot: TilePivot,
    /// The opacities of each **rendered** layer.
    /// 
    /// Only the top 4 layers will be rendered.
    pub layer_opacities: TilemapLayerOpacities,
    /// The storage of the tilemap. The entities of each tiles are divided into chunks and stored in it.
    /// 
    /// You need to spawn an empty tilemap and assign it to the storage.
    pub storage: TilemapStorage,
    /// The transform of the tilemap. It's not the same one as `Transform`.
    /// If you want to move or rotate the tilemap, you need to change this.
    /// Modify the `Transform` component will not work.
    pub tilemap_transform: TilemapTransform,
    /// The aabbs of the tilemap. Including the chunk aabbs and the tilemap aabb.
    pub aabbs: TilemapAabbs,
    /// Just to make sure the child sprites are correctly rendered.
    pub visibility: Visibility,
    /// Just to make sure the child sprites are correctly rendered.
    pub inherited_visibility: InheritedVisibility,
    /// Just to make sure the child sprites are correctly rendered.
    pub view_visibility: ViewVisibility,
    /// Modify `TilemapTransform` instead of this one.
    pub transform: Transform,
    /// Just to make sure the child sprites are correctly rendered.
    pub global_transform: GlobalTransform,
}
