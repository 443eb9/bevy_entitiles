use serde::{
    de::{Error, IgnoredAny, Visitor},
    Deserialize, Deserializer, Serialize,
};

use crate::{
    serializing::ldtk::json::LdtkColor, transfer_enum, transfer_enum_arr, transfer_field,
    unwrap_field,
};

use super::{definitions::TilesetRect, EntityRef, GridPoint};

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
                                return Err(Error::custom(format!("expected color, got {:?}", v)));
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
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum FieldValue {
    Integer(i32),
    Float(f32),
    Bool(bool),
    String(String),
    LocalEnum((String, String)),
    ExternEnum((String, String)),
    Color(LdtkColor),
    Point(GridPoint),
    EntityRef(EntityRef),

    IntegerArray(Vec<i32>),
    FloatArray(Vec<f32>),
    BoolArray(Vec<bool>),
    StringArray(Vec<String>),
    LocalEnumArray((String, Vec<String>)),
    ExternEnumArray((String, Vec<String>)),
    ColorArray(Vec<LdtkColor>),
    PointArray(Vec<GridPoint>),
    EntityRefArray(Vec<EntityRef>),
}
