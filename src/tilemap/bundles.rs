use bevy::ecs::bundle::Bundle;

use super::map::{
    TilePivot, TileRenderSize, TilemapAnimations, TilemapAxisFlip, TilemapLayerOpacities,
    TilemapName, TilemapSlotSize, TilemapStorage, TilemapTexture, TilemapTransform, TilemapType,
};

/// All the possible bundles of the tilemap.
#[derive(Debug, Clone)]
pub enum TilemapBundles {
    Data(DataTilemapBundle),
    PureColor(PureColorTilemapBundle),
    Texture(TilemapBundle),
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
    pub name: TilemapName,
    pub tile_render_size: TileRenderSize,
    pub slot_size: TilemapSlotSize,
    pub ty: TilemapType,
    pub tile_pivot: TilePivot,
    pub layer_opacities: TilemapLayerOpacities,
    pub storage: TilemapStorage,
    pub transform: TilemapTransform,
    pub axis_flip: TilemapAxisFlip,
    pub texture: TilemapTexture,
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
    pub name: TilemapName,
    pub tile_render_size: TileRenderSize,
    pub slot_size: TilemapSlotSize,
    pub ty: TilemapType,
    pub tile_pivot: TilePivot,
    pub layer_opacities: TilemapLayerOpacities,
    pub storage: TilemapStorage,
    pub transform: TilemapTransform,
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
