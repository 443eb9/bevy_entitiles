use serde::{Deserialize, Serialize};

use super::{definitions::TilesetRect, json::Nullable};

/*
 * Level
 */

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Level {
    #[serde(rename = "__bgColor")]
    pub bg_color: String,
    #[serde(rename = "__bgPos")]
    pub bg_pos: Nullable<ImagePosition>,
    #[serde(rename = "__neighbours")]
    pub neighbours: Vec<Neighbour>,
    pub bg_rel_path: Nullable<String>,
    pub external_rel_path: Nullable<String>,
    pub field_instances: Vec<FieldInstance>,
    pub identifier: String,
    pub iid: String,
    pub layer_instances: Vec<LayerInstance>,
    pub px_hei: i32,
    pub px_wid: i32,
    pub uid: i32,
    pub world_depth: i32,
    pub world_x: i32,
    pub world_y: i32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ImagePosition {
    pub crop_rect: [f32; 4],
    pub scale: [f32; 2],
    pub top_left_px: [i32; 2],
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Neighbour {
    pub dir: String,
    pub level_iid: String,
}

/*
 * Layer Instance
 */

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LayerInstance {
    #[serde(rename = "__cHei")]
    pub c_hei: i32,
    #[serde(rename = "__cWid")]
    pub c_wid: i32,
    #[serde(rename = "__gridSize")]
    pub grid_size: i32,
    #[serde(rename = "__identifier")]
    pub identifier: String,
    #[serde(rename = "__opacity")]
    pub opacity: f32,
    #[serde(rename = "__pxTotalOffsetX")]
    pub px_total_offset_x: i32,
    #[serde(rename = "__pxTotalOffsetY")]
    pub px_total_offset_y: i32,
    #[serde(rename = "__tilesetDefUid")]
    pub tileset_def_uid: Nullable<i32>,
    #[serde(rename = "__tilesetRelPath")]
    pub tileset_rel_path: Nullable<String>,
    #[serde(rename = "__type")]
    pub ty: String,
    pub auto_layer_tiles: Vec<TileInstance>,
    pub entity_instances: Vec<EntityInstance>,
    pub grid_tiles: Vec<TileInstance>,
    pub iid: String,
    pub int_grid_csv: Vec<i32>,
    pub layer_def_uid: i32,
    pub level_id: i32,
    pub override_tileset_uid: Nullable<i32>,
    pub px_offset_x: i32,
    pub px_offset_y: i32,
    pub visible: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TileInstance {
    #[serde(rename = "a")]
    pub alpha: f32,
    #[serde(rename = "f")]
    pub flip: i32,
    pub px: [i32; 2],
    pub src: [i32; 2],
    #[serde(rename = "t")]
    pub tile_id: i32,
}

/*
 * Entity Instance
 */

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EntityInstance {
    #[serde(rename = "__grid")]
    pub grid: [i32; 2],
    #[serde(rename = "__identifier")]
    pub identifier: String,
    #[serde(rename = "__pivot")]
    pub pivot: [f32; 2],
    #[serde(rename = "__smartColor")]
    pub smart_color: String,
    #[serde(rename = "__tags")]
    pub tags: Vec<String>,
    #[serde(rename = "__tile")]
    pub tile: Nullable<TilesetRect>,
    #[serde(rename = "__worldX")]
    pub world_x: i32,
    #[serde(rename = "__worldY")]
    pub world_y: i32,
    pub def_uid: i32,
    pub field_instances: Vec<FieldInstance>,
    pub iid: String,
    #[serde(rename = "px")]
    pub local_pos: [i32; 2],
    pub width: i32,
    pub height: i32,
}

/*
 * Field Instance
 */

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FieldInstance {
    pub def_uid: i32,
    #[serde(rename = "__type")]
    pub ty: String,
    #[serde(rename = "__value")]
    pub identifier: String,
    #[serde(rename = "__tile")]
    pub tile: Nullable<TilesetRect>,
    #[serde(rename = "__value")]
    pub value: FieldValue,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum FieldValue {
    Integer(i32),
    Float(f32),
    Bool(bool),
    String(String),
    Point(GridPoint),
    Tile(TilesetRect),
    EntityRef(EntityRef),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EntityRef {
    pub entity_id: String,
    pub layer_iid: String,
    pub level_iid: String,
    pub world_iid: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GridPoint {
    pub cx: i32,
    pub cy: i32,
}
