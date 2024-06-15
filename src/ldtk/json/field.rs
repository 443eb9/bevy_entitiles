use bevy::{math::IVec2, reflect::Reflect};
use serde::{
    de::{Error, IgnoredAny, Visitor},
    Deserialize, Deserializer, Serialize,
};

use crate::{
    ldtk::json::{definitions::TilesetRect, EntityRef, GridPoint, LdtkColor},
    match_field, match_field_enum, transfer_field, unwrap_field,
};

#[derive(Serialize, Debug, Clone, Reflect)]
#[serde(rename_all = "camelCase")]
pub struct FieldInstance {
    /// Reference of the Field definition UID
    pub def_uid: i32,

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
                        "defUid" => transfer_field!(def_uid, "defUid", map),
                        "__identifier" => {
                            transfer_field!(identifier, "__identifier", map)
                        }
                        "__tile" => transfer_field!(tile, "__tile", map),
                        "__type" => transfer_field!(ty, "__type", map),
                        "__value" => match ty.unwrap() {
                            "Int" => match_field!(value, Integer, i32, map),
                            "Float" => match_field!(value, Float, f32, map),
                            "Bool" => match_field!(value, Bool, bool, map),
                            "String" => match_field!(value, String, String, map),
                            "Multilines" => match_field!(value, String, String, map),
                            "FilePath" => match_field!(value, String, String, map),
                            "Color" => match_field!(value, Color, LdtkColor, map),
                            "Point" => match_field!(value, Point, GridPoint, map),
                            "EntityRef" => match_field!(value, EntityRef, EntityRef, map),
                            _ => {
                                let ty = ty.unwrap();
                                if ty.starts_with("LocalEnum") {
                                    match_field_enum!(
                                        value,
                                        LocalEnum,
                                        String,
                                        ty.split(".").nth(1).unwrap().to_string(),
                                        map
                                    );
                                } else if ty.starts_with("ExternEnum") {
                                    match_field_enum!(
                                        value,
                                        ExternEnum,
                                        String,
                                        ty.split(".").nth(1).unwrap().to_string(),
                                        map
                                    );
                                } else if ty.starts_with("Array") {
                                    let arr_ty =
                                        ty.split("<").nth(1).unwrap().split(">").nth(0).unwrap();
                                    if arr_ty.starts_with("LocalEnum") {
                                        match_field_enum!(
                                            value,
                                            LocalEnumArray,
                                            Vec<String>,
                                            arr_ty.split(".").nth(1).unwrap().to_string(),
                                            map
                                        );
                                    } else if arr_ty.starts_with("ExternEnum") {
                                        match_field_enum!(
                                            value,
                                            ExternEnumArray,
                                            Vec<String>,
                                            arr_ty.split(".").nth(1).unwrap().to_string(),
                                            map
                                        );
                                    } else {
                                        match arr_ty {
                                            "Int" => {
                                                match_field!(value, IntegerArray, Vec<i32>, map)
                                            }
                                            "Float" => {
                                                match_field!(value, FloatArray, Vec<f32>, map)
                                            }
                                            "Bool" => {
                                                match_field!(value, BoolArray, Vec<bool>, map)
                                            }
                                            "String" => {
                                                match_field!(value, StringArray, Vec<String>, map)
                                            }
                                            "Multilines" => {
                                                match_field!(value, StringArray, Vec<String>, map)
                                            }
                                            "FilePath" => {
                                                match_field!(value, StringArray, Vec<String>, map)
                                            }
                                            "Color" => {
                                                match_field!(value, ColorArray, Vec<LdtkColor>, map)
                                            }
                                            "Point" => {
                                                match_field!(value, PointArray, Vec<GridPoint>, map)
                                            }
                                            "EntityRef" => {
                                                match_field!(
                                                    value,
                                                    EntityRefArray,
                                                    Vec<EntityRef>,
                                                    map
                                                )
                                            }
                                            _ => {
                                                return Err(A::Error::custom(format!(
                                                    "Unknown type for field value: {}",
                                                    ty
                                                )));
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        _ => {
                            map.next_value::<IgnoredAny>()?;
                        }
                    }
                }

                let def_uid = unwrap_field!(def_uid, "defUid");
                let identifier = unwrap_field!(identifier, "__identifier");
                let tile = unwrap_field!(tile, "__tile");

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

/// - For classic types (ie. Integer, Float, Boolean, String, Text and FilePath), you just get the actual value with the expected type.
/// - For Color, the value is an hexadecimal string using "#rrggbb" format.
/// - For Enum, the value is a String representing the selected enum value.
/// - For Point, the value is a GridPoint object.
/// - For Tile, the value is a TilesetRect object.
/// - For EntityRef, the value is an EntityReferenceInfos object.
#[derive(Serialize, Deserialize, Debug, Clone, Reflect)]
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

macro_rules! impl_into {
    ($ty:ty, $variant:ident) => {
        impl Into<$ty> for FieldInstance {
            fn into(self) -> $ty {
                match self.value {
                    Some(v) => match v {
                        FieldValue::$variant(x) => x,
                        _ => panic!("Expected {} value!", stringify!($variant)),
                    },
                    None => panic!("Expected value!"),
                }
            }
        }
    };
}

macro_rules! impl_into_optional {
    ($ty:ty, $variant:ident) => {
        impl Into<Option<$ty>> for FieldInstance {
            fn into(self) -> Option<$ty> {
                match self.value {
                    Some(v) => match v {
                        FieldValue::$variant(x) => Some(x),
                        _ => panic!("Expected {} value!", stringify!($variant)),
                    },
                    None => None,
                }
            }
        }
    };
}

impl_into!(i32, Integer);
impl_into!(f32, Float);
impl_into!(bool, Bool);
impl_into!(String, String);
impl_into!(LdtkColor, Color);
impl_into!(GridPoint, Point);
impl_into!(EntityRef, EntityRef);

impl_into!(Vec<i32>, IntegerArray);
impl_into!(Vec<f32>, FloatArray);
impl_into!(Vec<bool>, BoolArray);
impl_into!(Vec<String>, StringArray);
impl_into!(Vec<LdtkColor>, ColorArray);
impl_into!(Vec<GridPoint>, PointArray);
impl_into!(Vec<EntityRef>, EntityRefArray);

impl_into_optional!(i32, Integer);
impl_into_optional!(f32, Float);
impl_into_optional!(bool, Bool);
impl_into_optional!(String, String);
impl_into_optional!(LdtkColor, Color);
impl_into_optional!(GridPoint, Point);
impl_into_optional!(EntityRef, EntityRef);

impl_into_optional!(Vec<i32>, IntegerArray);
impl_into_optional!(Vec<f32>, FloatArray);
impl_into_optional!(Vec<bool>, BoolArray);
impl_into_optional!(Vec<String>, StringArray);
impl_into_optional!(Vec<LdtkColor>, ColorArray);
impl_into_optional!(Vec<GridPoint>, PointArray);
impl_into_optional!(Vec<EntityRef>, EntityRefArray);

impl Into<IVec2> for FieldInstance {
    fn into(self) -> IVec2 {
        match self.value {
            Some(v) => match v {
                FieldValue::Point(p) => IVec2 { x: p.cx, y: p.cy },
                _ => panic!("Expected Point value!"),
            },
            None => panic!("Expected value!"),
        }
    }
}

impl Into<Option<IVec2>> for FieldInstance {
    fn into(self) -> Option<IVec2> {
        match self.value {
            Some(v) => match v {
                FieldValue::Point(p) => Some(IVec2 { x: p.cx, y: p.cy }),
                _ => panic!("Expected Point value!"),
            },
            None => None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_deser() {
        let json = r#"{
            "defUid": 1,
            "__identifier": "test",
            "__tile": null,
            "__type": "LocalEnum.Some",
            "__value": "A"
        }"#;

        let field_instance: Option<FieldInstance> = serde_json::from_str(json).unwrap();

        dbg!(field_instance);
    }
}
