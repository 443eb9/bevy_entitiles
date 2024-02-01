use bevy::ecs::bundle::Bundle;

use super::map::{
    TilePivot, TileRenderSize, TilemapAnimations, TilemapAxisFlip, TilemapLayerOpacities,
    TilemapName, TilemapSlotSize, TilemapStorage, TilemapTexture, TilemapTransform, TilemapType,
};

#[derive(Debug, Clone)]
pub enum TilemapBundles {
    Data(DataTilemapBundle),
    PureColor(PureColorTilemapBundle),
    Texture(TilemapBundle),
}

/// The bundle of the tilemap with no actual tiles.
#[derive(Bundle, Default, Debug, Clone)]
pub struct DataTilemapBundle {
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
    /// The axes for the tilemap.
    pub axis_direction: TilemapAxisFlip,
}

impl Into<TilemapBundle> for DataTilemapBundle {
    fn into(self) -> TilemapBundle {
        TilemapBundle {
            name: self.name,
            tile_render_size: self.tile_render_size,
            slot_size: self.slot_size,
            ty: self.ty,
            tile_pivot: self.tile_pivot,
            ..Default::default()
        }
    }
}

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
    pub transform: TilemapTransform,
    /// The axes for the tilemap.
    pub axis_flip: TilemapAxisFlip,
    /// The texture of the tilemap.
    pub texture: TilemapTexture,
    /// All the animation sequences of the tilemap.
    pub animations: TilemapAnimations,
}

impl Into<DataTilemapBundle> for TilemapBundle {
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

impl Into<PureColorTilemapBundle> for TilemapBundle {
    fn into(self) -> PureColorTilemapBundle {
        PureColorTilemapBundle {
            name: self.name,
            tile_render_size: self.tile_render_size,
            slot_size: self.slot_size,
            ty: self.ty,
            tile_pivot: self.tile_pivot,
            layer_opacities: self.layer_opacities,
            storage: self.storage,
            transform: self.transform,
            axis_flip: self.axis_flip,
        }
    }
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
    pub transform: TilemapTransform,
    /// The axes for the tilemap.
    pub axis_flip: TilemapAxisFlip,
}

impl PureColorTilemapBundle {
    pub fn convert_to_texture_bundle(
        self,
        texture: TilemapTexture,
        animations: TilemapAnimations,
    ) -> TilemapBundle {
        TilemapBundle {
            name: self.name,
            tile_render_size: self.tile_render_size,
            slot_size: self.slot_size,
            ty: self.ty,
            tile_pivot: self.tile_pivot,
            layer_opacities: self.layer_opacities,
            storage: self.storage,
            transform: self.transform,
            axis_flip: self.axis_flip,
            texture,
            animations,
        }
    }
}
