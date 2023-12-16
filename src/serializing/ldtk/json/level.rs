use bevy::reflect::Reflect;
use serde::{
    de::{Error, IgnoredAny, Visitor},
    Deserialize, Deserializer, Serialize,
};

use crate::{transfer_enum, transfer_enum_arr, transfer_field, unwrap_field};

use super::{
    definitions::{LayerType, TilesetRect},
    EntityRef, GridPoint, LdtkColor,
};

/*
 * Level
 */

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Level {
    /// Background color of the level (same as `bgColor`, except
    /// the default value is automatically used here if its value is `null`)
    #[serde(rename = "__bgColor")]
    pub bg_color: LdtkColor,

    /// Position informations of the background image, if there is one.
    #[serde(rename = "__bgPos")]
    pub bg_pos: Option<ImagePosition>,

    /// An array listing all other levels touching this one on the world map.
    /// Since 1.4.0, this includes levels that overlap in the same world layer,
    /// or in nearby world layers.
    ///
    /// Only relevant for world layouts where level spatial positioning is manual
    /// (ie. GridVania, Free). For Horizontal and Vertical layouts,
    /// this array is always empty.
    #[serde(rename = "__neighbours")]
    pub neighbours: Vec<Neighbour>,

    /// The optional relative path to the level background image.
    pub bg_rel_path: Option<String>,

    /// This value is not null if the project option
    /// "Save levels separately" is enabled. In this case,
    /// this relative path points to the level Json file.
    pub external_rel_path: Option<String>,

    /// An array containing this level custom field values.
    pub field_instances: Vec<FieldInstance>,

    /// User defined unique identifier
    pub identifier: String,

    /// Unique instance identifier
    pub iid: String,

    /// An array containing all Layer instances.
    /// ## IMPORTANT:
    /// if the project option "Save levels separately" is enabled,
    /// this field will be null.
    ///
    /// This array is **sorted in display order**: the 1st layer is
    /// the top-most and the last is behind.
    pub layer_instances: Vec<LayerInstance>,

    /// Height of the level in pixels
    pub px_hei: i32,

    /// Width of the level in pixels
    pub px_wid: i32,

    /// Unique Int identifier
    pub uid: i32,

    /// Index that represents the "depth" of the level in the world.
    /// Default is 0, greater means "above", lower means "below".
    ///
    /// This value is mostly used for display only and is intended to
    /// make stacking of levels easier to manage.
    pub world_depth: i32,

    /// World X coordinate in pixels.
    ///
    /// Only relevant for world layouts where level spatial positioning is manual
    /// (ie. GridVania, Free).
    /// For Horizontal and Vertical layouts, the value is always -1 here.
    pub world_x: i32,

    /// World Y coordinate in pixels.
    ///
    /// Only relevant for world layouts where level spatial positioning is manual
    /// (ie. GridVania, Free).
    /// For Horizontal and Vertical layouts, the value is always -1 here.
    pub world_y: i32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ImagePosition {
    /// An array of 4 float values describing the cropped sub-rectangle
    /// of the displayed background image. This cropping happens when
    /// original is larger than the level bounds
    ///
    /// Array format: `[ cropX, cropY, cropWidth, cropHeight ]`
    pub crop_rect: [f32; 4],

    /// An array containing the `[scaleX,scaleY]` values of the cropped
    /// background image, depending on `bgPos` option.
    pub scale: [f32; 2],

    /// An array containing the `[x,y]` pixel coordinates of the top-left
    /// corner of the cropped background image, depending on `bgPos` option.
    pub top_left_px: [i32; 2],
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Neighbour {
    /// A single lowercase character tipping on the level location
    /// (`n`orth, `s`outh, `w`est, `e`ast).
    ///
    /// Since 1.4.0, this character value can also be
    /// `<` (neighbour depth is lower),
    /// `>` (neighbour depth is greater)
    /// or `o` (levels overlap and share the same world depth).
    pub dir: String,

    /// Neighbour Instance Identifier
    pub level_iid: String,
}

/*
 * Layer Instance
 */

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LayerInstance {
    /// Grid-based height
    #[serde(rename = "__cHei")]
    pub c_hei: i32,

    /// Grid-based width
    #[serde(rename = "__cWid")]
    pub c_wid: i32,

    /// Grid size
    #[serde(rename = "__gridSize")]
    pub grid_size: i32,

    /// Layer definition identifier
    #[serde(rename = "__identifier")]
    pub identifier: String,

    /// Layer opacity as Float [0-1]
    #[serde(rename = "__opacity")]
    pub opacity: f32,

    ///	Total layer X pixel offset, including both instance and definition offsets.
    #[serde(rename = "__pxTotalOffsetX")]
    pub px_total_offset_x: i32,

    /// Total layer Y pixel offset, including both instance and definition offsets.
    #[serde(rename = "__pxTotalOffsetY")]
    pub px_total_offset_y: i32,

    /// The definition UID of corresponding Tileset, if any.
    #[serde(rename = "__tilesetDefUid")]
    pub tileset_def_uid: Option<i32>,

    /// The relative path to corresponding Tileset, if any.
    #[serde(rename = "__tilesetRelPath")]
    pub tileset_rel_path: Option<String>,

    /// Layer type (possible values: IntGrid, Entities, Tiles or AutoLayer)
    #[serde(rename = "__type")]
    pub ty: LayerType,

    /// An array containing all tiles generated by Auto-layer rules.
    /// The array is already sorted in display order
    /// (ie. 1st tile is beneath 2nd, which is beneath 3rd etc.).
    ///
    /// Note: if multiple tiles are stacked in the same cell as the result of different rules,
    /// all tiles behind opaque ones will be discarded.
    pub auto_layer_tiles: Vec<TileInstance>,
    pub entity_instances: Vec<EntityInstance>,
    pub grid_tiles: Vec<TileInstance>,

    /// Unique layer instance identifier
    pub iid: String,

    /// A list of all values in the IntGrid layer, stored in CSV format (Comma Separated Values).
    ///
    /// Order is from left to right, and top to bottom (ie. first row from left to right, followed by second row, etc).
    ///
    /// `0` means "empty cell" and IntGrid values start at 1.
    ///
    /// The array size is `__cWid` x `__cHei` cells.
    pub int_grid_csv: Vec<i32>,

    /// Reference the Layer definition UID
    pub layer_def_uid: i32,

    /// Reference to the UID of the level containing this layer instance
    pub level_id: i32,

    /// This layer can use another tileset by overriding the tileset UID here.
    pub override_tileset_uid: Option<i32>,

    /// X offset in pixels to render this layer, usually 0
    /// ## IMPORTANT:
    /// this should be added to the LayerDef optional offset,
    /// so you should probably prefer using `__pxTotalOffsetX`
    /// which contains the total offset value)
    pub px_offset_x: i32,
    pub px_offset_y: i32,
    pub visible: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TileInstance {
    ///	Alpha/opacity of the tile (0-1, defaults to 1)
    #[serde(rename = "a")]
    pub alpha: f32,

    /// "Flip bits", a 2-bits integer to represent the mirror transformations of the tile.
    /// - Bit 0 = X flip
    /// - Bit 1 = Y flip
    ///
    /// Examples: f=0 (no flip), f=1 (X flip only), f=2 (Y flip only), f=3 (both flips)
    ///
    /// (This is the same as the `TileFlip`)
    #[serde(rename = "f")]
    pub flip: i32,

    /// Pixel coordinates of the tile in the layer (`[x,y]` format).
    /// Don't forget optional layer offsets, if they exist!
    pub px: [i32; 2],

    /// Pixel coordinates of the tile in the tileset ([x,y] format)
    pub src: [i32; 2],

    /// The Tile ID in the corresponding tileset.
    #[serde(rename = "t")]
    pub tile_id: i32,
}

/*
 * Entity Instance
 */

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EntityInstance {
    /// Grid-based coordinates (`[x,y]` format)
    #[serde(rename = "__grid")]
    pub grid: [i32; 2],

    /// Entity definition identifier
    #[serde(rename = "__identifier")]
    pub identifier: String,

    /// Pivot coordinates (`[x,y]` format, values are from 0 to 1) of the Entity
    #[serde(rename = "__pivot")]
    pub pivot: [f32; 2],

    /// The entity "smart" color, guessed from either Entity definition,
    /// or one its field instances.
    #[serde(rename = "__smartColor")]
    pub smart_color: LdtkColor,

    /// Array of tags defined in this Entity definition.
    #[serde(rename = "__tags")]
    pub tags: Vec<String>,

    /// Optional TilesetRect used to display this entity
    /// (it could either be the default Entity tile,
    /// or some tile provided by a field value, like an Enum).
    #[serde(rename = "__tile")]
    pub tile: Option<TilesetRect>,

    /// X world coordinate in pixels
    #[serde(rename = "__worldX")]
    pub world_x: i32,

    /// Y world coordinate in pixels
    #[serde(rename = "__worldY")]
    pub world_y: i32,

    /// Reference of the Entity definition UID
    pub def_uid: i32,

    /// An array of all custom fields and their values.
    pub field_instances: Vec<FieldInstance>,

    /// Unique instance identifier
    pub iid: String,

    /// Pixel coordinates (`[x,y]` format) in current level coordinate space.
    /// Don't forget optional layer offsets, if they exist!
    #[serde(rename = "px")]
    pub local_pos: [i32; 2],

    /// Entity width in pixels.
    /// For non-resizable entities, it will be the same as Entity definition.
    pub width: i32,

    /// Entity height in pixels.
    /// For non-resizable entities, it will be the same as Entity definition.
    pub height: i32,
}

/*
 * Field Instance
 */

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FieldInstance {
    /// Reference of the Field definition UID
    pub def_uid: i32,

    /// Type of the field, such as Int, Float, String, Enum(my_enum_name), Bool, etc.
    ///
    /// NOTE: if you enable the advanced option Use Multilines type,
    /// you will have "Multilines" instead of "String" when relevant.
    ///
    /// This is not required because we can use enum.
    /// So the type of the `value` = `type`
    /// #[serde(rename = "__type")]
    /// pub ty: FieldType,

    /// Field definition identifier
    #[serde(rename = "__identifier")]
    pub identifier: String,

    /// Optional TilesetRect used to display this field
    /// (this can be the field own Tile,
    /// or some other Tile guessed from the value, like an Enum).
    #[serde(rename = "__tile")]
    pub tile: Option<TilesetRect>,

    /// Actual value of the field instance. The value type varies, depending on `__type`
    /// If the field is an array, then this `__value` will also be a JSON array.
    #[serde(rename = "__value")]
    pub value: Option<FieldValue>,
}

const FIELDS: &[&str] = &["defUid", "__identifier", "__tile", "__type", "__value"];

impl<'de> Deserialize<'de> for FieldInstance {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        pub struct FieldInstanceVisitor;
        impl<'de> Visitor<'de> for FieldInstanceVisitor {
            type Value = FieldInstance;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a field instance")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut def_uid = None;
                let mut identifier = None;
                let mut tile = None;
                let mut ty = None;
                let mut value = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        FieldInstanceFields::DefUid => transfer_field!(def_uid, "defUid", map),
                        FieldInstanceFields::Identifier => {
                            transfer_field!(identifier, "__identifier", map)
                        }
                        FieldInstanceFields::Tile => transfer_field!(tile, "__tile", map),
                        FieldInstanceFields::Type => transfer_field!(ty, "__type", map),
                        FieldInstanceFields::Value => transfer_field!(value, "__value", map),
                        FieldInstanceFields::Skip => {
                            map.next_value::<IgnoredAny>()?;
                        }
                    }
                }

                let def_uid = unwrap_field!(def_uid, "defUid");
                let identifier = unwrap_field!(identifier, "__identifier");
                let tile = unwrap_field!(tile, "__tile");
                let ty = unwrap_field!(ty, "__type");
                let value = unwrap_field!(value, "__value");

                let value = match ty {
                    SpecialFieldType::LocalEnum(name) => {
                        transfer_enum!(LocalEnum, "local enum", name, value)
                    }
                    SpecialFieldType::ExternEnum(name) => {
                        transfer_enum!(ExternEnum, "extern enum", name, value)
                    }
                    SpecialFieldType::Color => {
                        if let Some(v) = value {
                            if let FieldValue::String(s) = v {
                                Some(FieldValue::Color(LdtkColor::from(s)))
                            } else {
                                return Err(A::Error::custom(format!(
                                    "expected color, got {:?}",
                                    v
                                )));
                            }
                        } else {
                            None
                        }
                    }
                    SpecialFieldType::LocalEnumArray(name) => {
                        transfer_enum_arr!(LocalEnumArray, "string array", name, value)
                    }
                    SpecialFieldType::ExternEnumArray(name) => {
                        transfer_enum_arr!(ExternEnumArray, "string array", name, value)
                    }
                    SpecialFieldType::None => value,
                };

                Ok(FieldInstance {
                    def_uid,
                    identifier,
                    tile,
                    value,
                })
            }
        }

        deserializer.deserialize_struct("FieldInstance", FIELDS, FieldInstanceVisitor)
    }
}

pub enum FieldInstanceFields {
    DefUid,
    Identifier,
    Tile,
    Type,
    Value,
    Skip,
}

impl<'de> Deserialize<'de> for FieldInstanceFields {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        pub struct FieldInstanceFieldsVisitor;
        impl<'de> Visitor<'de> for FieldInstanceFieldsVisitor {
            type Value = FieldInstanceFields;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a field instance field")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match v {
                    "defUid" => Ok(FieldInstanceFields::DefUid),
                    "__identifier" => Ok(FieldInstanceFields::Identifier),
                    "__tile" => Ok(FieldInstanceFields::Tile),
                    "__type" => Ok(FieldInstanceFields::Type),
                    "__value" => Ok(FieldInstanceFields::Value),
                    _ => Ok(FieldInstanceFields::Skip),
                }
            }
        }

        deserializer.deserialize_identifier(FieldInstanceFieldsVisitor)
    }
}

#[derive(Serialize, Debug)]
pub enum SpecialFieldType {
    None,
    LocalEnum(String),
    ExternEnum(String),

    LocalEnumArray(String),
    ExternEnumArray(String),
    Color,
}

impl<'de> Deserialize<'de> for SpecialFieldType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        pub struct FieldTypeVisitor;
        impl<'de> Visitor<'de> for FieldTypeVisitor {
            type Value = SpecialFieldType;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a field type")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if v.starts_with("LocalEnum") {
                    return Ok(SpecialFieldType::LocalEnum(
                        v.split(".").last().unwrap().to_string(),
                    ));
                }
                if v.starts_with("ExternEnum") {
                    return Ok(SpecialFieldType::ExternEnum(
                        v.split(".").last().unwrap().to_string(),
                    ));
                }
                if v.starts_with("Array") {
                    let ty = v.split("<").nth(1).unwrap().split(">").nth(0).unwrap();
                    if ty.starts_with("LocalEnum") {
                        return Ok(SpecialFieldType::LocalEnumArray(
                            ty.split(".").last().unwrap().to_string(),
                        ));
                    }
                    if ty.starts_with("ExternEnum") {
                        return Ok(SpecialFieldType::ExternEnumArray(
                            ty.split(".").last().unwrap().to_string(),
                        ));
                    }
                }

                match v {
                    "Color" => Ok(SpecialFieldType::Color),
                    _ => Ok(SpecialFieldType::None),
                }
            }
        }

        deserializer.deserialize_str(FieldTypeVisitor)
    }
}

/// - For classic types (ie. Integer, Float, Boolean, String, Text and FilePath), you just get the actual value with the expected type.
/// - For Color, the value is an hexadecimal string using "#rrggbb" format.
/// - For Enum, the value is a String representing the selected enum value.
/// - For Point, the value is a GridPoint object.
/// - For Tile, the value is a TilesetRect object.
/// - For EntityRef, the value is an EntityReferenceInfos object.
#[derive(Serialize, Deserialize, Debug, Clone, Reflect)]
#[serde(untagged)]
pub enum FieldValue {
    IntegerArray(Vec<i32>),
    FloatArray(Vec<f32>),
    BoolArray(Vec<bool>),
    StringArray(Vec<String>),
    LocalEnumArray((String, Vec<String>)),
    ExternEnumArray((String, Vec<String>)),
    ColorArray(Vec<LdtkColor>),
    PointArray(Vec<GridPoint>),
    EntityRefArray(Vec<EntityRef>),

    Integer(i32),
    Float(f32),
    Bool(bool),
    String(String),
    LocalEnum((String, String)),
    ExternEnum((String, String)),
    Color(LdtkColor),
    Point(GridPoint),
    EntityRef(EntityRef),
}
