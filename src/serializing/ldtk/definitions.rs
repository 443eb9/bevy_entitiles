use serde::{Deserialize, Serialize};

use super::json::Nullable;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Definitions {
    pub tilesets: Vec<TilesetDef>,
    pub layers: Vec<LayerDef>,
    pub enums: Vec<EnumDef>,
    pub entities: Vec<EntityDef>,
    pub external_enums: Vec<EnumDef>,
}

/*
 * Layer Definition
 */

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LayerDef {
    #[serde(rename = "__type")]
    pub ty: String,
    pub auto_source_layer_def_uid: Nullable<i32>,
    pub display_opacity: f32,
    pub grid_size: i32,
    pub identifier: String,
    pub int_grid_values: Vec<IntGridValue>,
    pub int_grid_values_groups: Vec<IntGroupValueGroup>,
    pub parallax_factor_x: f32,
    pub parallax_factor_y: f32,
    pub parallax_scaling: bool,
    pub px_offset_x: i32,
    pub px_offset_y: i32,
    pub tileset_def_uid: Nullable<i32>,
    pub uid: i32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IntGridValue {
    pub color: String,
    pub group_uid: i32,
    pub identifier: Nullable<String>,
    pub tile: Nullable<TilesetRect>,
    pub value: i32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IntGroupValueGroup {
    pub color: Nullable<String>,
    pub identifier: Nullable<String>,
    pub uid: i32,
}

/*
 * Entity Definition
 */

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EntityDef {
    pub color: String,
    pub identifier: String,
    pub nine_slice_borders: [i32; 4],
    pub pivot_x: f32,
    pub pivot_y: f32,
    pub tile_rect: Nullable<TilesetRect>,
    pub tile_render_mode: TileRenderMode,
    pub tileset_id: Nullable<i32>,
    pub ui_tile_rect: Nullable<TilesetRect>,
    pub uid: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum TileRenderMode {
    Cover,
    FitInside,
    Repeat,
    Stretch,
    FullSizeCropped,
    FullSizeUncropped,
    NineSlice,
}

/*
 * Tileset Definition
 */

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TilesetDef {
    #[serde(rename = "__cHei")]
    pub c_hei: i32,
    #[serde(rename = "__cWid")]
    pub c_wid: i32,
    pub custom_data: Vec<CustomData>,
    pub embed_atlas: Nullable<String>,
    pub enum_tags: Vec<EnumTag>,
    pub identifier: String,
    pub padding: i32,
    pub px_hei: i32,
    pub px_wid: i32,
    pub rel_path: Nullable<String>,
    pub spacing: i32,
    pub tags: Vec<String>,
    pub tags_source_enum_uid: Nullable<i32>,
    pub tile_grid_size: i32,
    pub uid: i32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CustomData {
    pub data: String,
    pub tile_id: i32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EnumTag {
    pub enum_value_id: String,
    pub tile_ids: Vec<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TilesetRect {
    pub tileset_uid: i32,
    #[serde(rename = "x")]
    pub x_pos: i32,
    #[serde(rename = "y")]
    pub y_pos: i32,
    #[serde(rename = "w")]
    pub width: i32,
    #[serde(rename = "h")]
    pub height: i32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EnumTagValue {
    pub tile_ids: Vec<i32>,
    pub enum_value_id: String,
}

/*
 * Enum Definition
 */

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EnumDef {
    pub external_rel_path: Nullable<String>,
    pub icon_tileset_uid: Nullable<String>,
    pub identifier: String,
    pub tags: Vec<String>,
    pub uid: i32,
    pub values: Vec<EnumValue>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EnumValue {
    pub color: String,
    pub id: String,
    pub tile_rect: Nullable<TilesetRect>,
}
