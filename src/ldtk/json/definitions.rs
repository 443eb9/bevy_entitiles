use bevy::reflect::Reflect;
use serde::{de::Visitor, Deserialize, Serialize};

use crate::ldtk::sprite::{NineSliceBorders, TileRenderMode};

#[derive(Serialize, Deserialize, Debug, Reflect)]
#[serde(rename_all = "camelCase")]
pub struct Definitions {
    /// All entities definitions, including their custom fields
    pub entities: Vec<EntityDef>,

    /// All internal enums
    pub enums: Vec<EnumDef>,

    /// Note: external enums are exactly the same as enums,
    /// except they have a relPath to point to an external source file.
    pub external_enums: Vec<EnumDef>,

    /// All layer definitions
    pub layers: Vec<LayerDef>,

    /// All custom fields available to all levels.
    pub tilesets: Vec<TilesetDef>,
}

/*
 * Layer Definition
 */

#[derive(Serialize, Deserialize, Debug, Reflect)]
#[serde(rename_all = "camelCase")]
pub struct LayerDef {
    /// Type of the layer (IntGrid, Entities, Tiles or AutoLayer)
    #[serde(rename = "__type")]
    pub ty: LayerType,

    pub auto_source_layer_def_uid: Option<i32>,

    /// Opacity of the layer (0 to 1.0)
    pub display_opacity: f32,

    /// Width and height of the grid in pixels
    pub grid_size: i32,

    /// User defined unique identifier
    pub identifier: String,

    /// An array that defines extra optional info for each IntGrid value.
    /// ## WARNING:
    /// the array order is not related to actual IntGrid values!
    /// As user can re-order IntGrid values freely, you may value "2" before value "1" in this array.
    pub int_grid_values: Vec<IntGridValue>,

    /// Group informations for IntGrid values
    pub int_grid_values_groups: Vec<IntGroupValueGroup>,
    pub parallax_factor_x: f32,
    pub parallax_factor_y: f32,
    pub parallax_scaling: bool,
    pub px_offset_x: i32,
    pub px_offset_y: i32,
    pub tileset_def_uid: Option<i32>,
    pub uid: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum LayerType {
    IntGrid,
    Entities,
    Tiles,
    AutoLayer,
}

#[derive(Serialize, Deserialize, Debug, Reflect)]
#[serde(rename_all = "camelCase")]
pub struct IntGridValue {
    pub color: String,
    pub group_uid: i32,
    pub identifier: Option<String>,
    pub tile: Option<TilesetRect>,
    pub value: i32,
}

#[derive(Serialize, Deserialize, Debug, Reflect)]
#[serde(rename_all = "camelCase")]
pub struct IntGroupValueGroup {
    /// User defined color
    pub color: Option<String>,

    /// User defined string identifier
    pub identifier: Option<String>,

    /// Group unique ID
    pub uid: i32,
}

/*
 * Entity Definition
 */

#[derive(Serialize, Deserialize, Debug, Clone, Reflect)]
#[serde(rename_all = "camelCase")]
pub struct EntityDef {
    /// Base entity color
    pub color: String,

    /// User defined unique identifier
    pub identifier: String,

    /// An array of 4 dimensions for the up/right/down/left borders (in this order)
    /// when using 9-slice mode for tileRenderMode.
    /// If the tileRenderMode is not NineSlice, then this array is empty.
    /// See: https://en.wikipedia.org/wiki/9-slice_scaling
    pub nine_slice_borders: NineSliceBorders,

    /// Pivot X coordinate (from 0 to 1.0)
    pub pivot_x: f32,

    /// Pivot Y coordinate (from 0 to 1.0)
    pub pivot_y: f32,

    /// An object representing a rectangle from an existing Tileset
    pub tile_rect: Option<TilesetRect>,

    /// An enum describing how the the Entity tile is rendered inside the Entity bounds.
    pub tile_render_mode: TileRenderMode,

    /// Tileset ID used for optional tile display
    pub tileset_id: Option<i32>,

    /// This tile overrides the one defined in tileRect in the UI
    pub ui_tile_rect: Option<TilesetRect>,

    /// Unique Int identifier
    pub uid: i32,

    /// Pixel width
    pub width: i32,

    /// Pixel height
    pub height: i32,
}

impl<'de> Deserialize<'de> for NineSliceBorders {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(NineSliceBordersVisitor)
    }
}

pub struct NineSliceBordersVisitor;

impl<'de> Visitor<'de> for NineSliceBordersVisitor {
    type Value = NineSliceBorders;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an array of [top, right, bottom, left] values")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut borders = NineSliceBorders {
            is_valid: false,
            up: 0,
            right: 0,
            down: 0,
            left: 0,
        };

        if let Some(x) = seq.next_element()? {
            borders.up = x;
            borders.is_valid = true;
        } else {
            return Ok(borders);
        }

        borders.right = seq.next_element()?.unwrap();
        borders.down = seq.next_element()?.unwrap();
        borders.left = seq.next_element()?.unwrap();

        Ok(borders)
    }
}

/*
 * Tileset Definition
 */

#[derive(Serialize, Deserialize, Debug, Reflect)]
#[serde(rename_all = "camelCase")]
pub struct TilesetDef {
    /// Grid-based height
    #[serde(rename = "__cHei")]
    pub c_hei: i32,

    /// Grid-based width
    #[serde(rename = "__cWid")]
    pub c_wid: i32,

    /// An array of custom tile metadata
    pub custom_data: Vec<CustomData>,

    /// If this value is set, then it means that this atlas uses
    /// an internal LDtk atlas image instead of a loaded one.
    pub embed_atlas: Option<String>,

    /// Tileset tags using Enum values specified by `tagsSourceEnumId`.
    /// This array contains 1 element per Enum value,
    /// which contains an array of all Tile IDs that are tagged with it.
    pub enum_tags: Vec<EnumTag>,

    /// User defined unique identifier
    pub identifier: String,

    /// Distance in pixels from image borders
    pub padding: i32,

    /// Image height in pixels
    pub px_hei: i32,

    /// Image width in pixels
    pub px_wid: i32,

    /// Path to the source file, relative to the current project JSON file
    /// It can be null if no image was provided, or when using an embed atlas.
    pub rel_path: Option<String>,

    /// Space in pixels between all tiles
    pub spacing: i32,

    /// An array of user-defined tags to organize the Tilesets
    pub tags: Vec<String>,

    /// Optional Enum definition UID used for this tileset meta-data
    pub tags_source_enum_uid: Option<i32>,

    pub tile_grid_size: i32,

    /// Unique Intidentifier
    pub uid: i32,
}

#[derive(Serialize, Deserialize, Debug, Reflect)]
#[serde(rename_all = "camelCase")]
pub struct CustomData {
    pub data: String,
    pub tile_id: i32,
}

#[derive(Serialize, Deserialize, Debug, Reflect)]
#[serde(rename_all = "camelCase")]
pub struct EnumTag {
    pub enum_value_id: String,
    pub tile_ids: Vec<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect)]
#[serde(rename_all = "camelCase")]
pub struct TilesetRect {
    /// UID of the tileset
    pub tileset_uid: i32,

    /// X pixels coordinate of the top-left corner in the Tileset image
    #[serde(rename = "x")]
    pub x_pos: i32,

    /// Y pixels coordinate of the top-left corner in the Tileset image
    #[serde(rename = "y")]
    pub y_pos: i32,

    /// Width in pixels
    #[serde(rename = "w")]
    pub width: i32,

    /// Height in pixels
    #[serde(rename = "h")]
    pub height: i32,
}

#[derive(Serialize, Deserialize, Debug, Reflect)]
#[serde(rename_all = "camelCase")]
pub struct EnumTagValue {
    pub tile_ids: Vec<i32>,
    pub enum_value_id: String,
}

/*
 * Enum Definition
 */

#[derive(Serialize, Deserialize, Debug, Reflect)]
#[serde(rename_all = "camelCase")]
pub struct EnumDef {
    /// Relative path to the external file providing this Enum
    pub external_rel_path: Option<String>,

    /// Tileset UID if provided
    pub icon_tileset_uid: Option<i32>,

    /// User defined unique identifier
    pub identifier: String,

    /// An array of user-defined tags to organize the Enums
    pub tags: Vec<String>,

    /// Unique Int identifier
    pub uid: i32,

    /// All possible enum values, with their optional Tile infos.
    pub values: Vec<EnumValue>,
}

#[derive(Serialize, Deserialize, Debug, Reflect)]
#[serde(rename_all = "camelCase")]
pub struct EnumValue {
    /// Optional color
    pub color: i32,

    /// Enum value
    pub id: String,

    /// Optional tileset rectangle to represents this value
    pub tile_rect: Option<TilesetRect>,
}
