use bevy::{reflect::Reflect, render::color::Color, utils::HashMap};
use serde::{
    de::{IgnoredAny, Visitor},
    Deserialize, Serialize,
};

use super::TiledColor;

#[derive(Debug, Default, Clone, Reflect, Serialize, Deserialize)]
pub struct Components {
    #[serde(rename = "property")]
    pub instances: Vec<ClassInstance>,
}

#[derive(Debug, Clone, Reflect, Serialize)]
pub struct ClassInstance {
    pub name: String,
    pub ty: String,
    pub properties: HashMap<String, PropertyInstance>,
}

impl<'de> Deserialize<'de> for ClassInstance {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct PropertiesWrapper {
            #[serde(rename = "property")]
            properties: Vec<PropertyInstance>,
        }

        struct ClassInstanceVisitor;
        impl<'de> Visitor<'de> for ClassInstanceVisitor {
            type Value = ClassInstance;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a class instance")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut name = None;
                let mut ty = None;
                let mut properties = None;
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "@name" => {
                            name = Some(map.next_value::<String>()?);
                        }
                        "@type" => {
                            map.next_value::<IgnoredAny>()?;
                        }
                        "@propertytype" => {
                            ty = Some(map.next_value::<String>()?);
                        }
                        "properties" => {
                            properties = Some(
                                map.next_value::<PropertiesWrapper>()?
                                    .properties
                                    .into_iter()
                                    .map(|prop| (prop.name.clone(), prop))
                                    .collect(),
                            );
                        }
                        "@value" => {
                            panic!("Primitive properties are not allowed in ClassInstance (name={})", name.unwrap_or("UNK".to_owned()));
                        }
                        _ => panic!("Unknown key for ClassInstance: {}", key),
                    }
                }

                Ok(ClassInstance {
                    name: name.unwrap(),
                    ty: ty.unwrap(),
                    properties: properties.unwrap_or_default(),
                })
            }
        }

        deserializer.deserialize_map(ClassInstanceVisitor)
    }
}

#[derive(Debug, Clone, Reflect, Serialize)]
pub struct PropertyInstance {
    pub name: String,
    pub ty: String,
    pub value: PropertyValue,
}

impl<'de> Deserialize<'de> for PropertyInstance {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct PropertyInstanceVisitor;
        impl<'de> Visitor<'de> for PropertyInstanceVisitor {
            type Value = PropertyInstance;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a property instance")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut name = None;
                let mut ty = "string".to_string();
                let mut value = None;
                let mut enum_ty = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "@name" => {
                            name = Some(map.next_value::<String>()?);
                        }
                        "@type" => {
                            ty = map.next_value::<String>()?;
                        }
                        "@value" => match ty.as_str() {
                            "int" => {
                                value = Some(PropertyValue::Int(map.next_value::<i32>()?));
                            }
                            "float" => {
                                value = Some(PropertyValue::Float(map.next_value::<f32>()?));
                            }
                            "bool" => {
                                value = Some(PropertyValue::Bool(map.next_value::<bool>()?));
                            }
                            "string" | "file" => {
                                value = Some(PropertyValue::String(map.next_value::<String>()?));
                            }
                            "color" => {
                                value = Some(PropertyValue::Color(map.next_value::<TiledColor>()?));
                            }
                            "object" => {
                                value = Some(PropertyValue::ObjectRef(map.next_value::<u32>()?));
                            }
                            _ => {
                                panic!(
                                    "Seems like there is a nested custom class type {} \
                                    in the property {} which is not supported yet.",
                                    map.next_value::<String>()?,
                                    name.unwrap()
                                );
                            }
                        },
                        "@propertytype" => {
                            enum_ty = Some(map.next_value::<String>()?);
                        }
                        "property" => {
                            return map.next_value::<PropertyInstance>();
                        }
                        _ => panic!("Unknown key for PropertyInstance: {}", key),
                    }
                }

                if let Some(enum_name) = enum_ty {
                    let PropertyValue::String(variant) = value.unwrap() else {
                        unreachable!()
                    };

                    value = Some(PropertyValue::Enum(enum_name, variant));
                }

                Ok(PropertyInstance {
                    name: name.unwrap(),
                    ty,
                    value: value.unwrap(),
                })
            }
        }

        deserializer.deserialize_map(PropertyInstanceVisitor)
    }
}

macro_rules! impl_into {
    ($ty:ty, $variant:ident) => {
        impl Into<$ty> for PropertyInstance {
            fn into(self) -> $ty {
                match self.value {
                    PropertyValue::$variant(x) => x,
                    _ => panic!("Expected {} value!", stringify!($variant)),
                }
            }
        }
    };
}

impl_into!(i32, Int);
impl_into!(f32, Float);
impl_into!(bool, Bool);
impl_into!(String, String);
impl_into!(TiledColor, Color);
impl_into!(u32, ObjectRef);

impl Into<Color> for PropertyInstance {
    fn into(self) -> Color {
        match self.value {
            PropertyValue::Color(x) => x.into(),
            _ => panic!("Expected Color value!"),
        }
    }
}

#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub enum PropertyValue {
    Int(i32),
    Float(f32),
    Bool(bool),
    String(String),
    Color(TiledColor),
    Enum(String, String),
    ObjectRef(u32),
}
