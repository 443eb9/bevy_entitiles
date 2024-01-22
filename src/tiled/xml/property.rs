use bevy::utils::HashMap;
use serde::{
    de::{IgnoredAny, Visitor},
    Deserialize, Serialize,
};

use super::TiledColor;

pub type FieldIdentifier = String;

#[derive(Debug, Default, Clone, Serialize)]
pub struct Components(HashMap<String, ClassInstance>);

impl<'de> Deserialize<'de> for Components {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct CustomPropertiesVisitor;
        impl<'de> Visitor<'de> for CustomPropertiesVisitor {
            type Value = Components;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a custom property")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut components = HashMap::default();
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "property" => {
                            let component = map.next_value::<ClassInstance>()?;
                            components.insert(component.name.clone(), component);
                        }
                        _ => panic!("Unknown key: {}", key),
                    }
                }

                Ok(Components(components))
            }
        }

        deserializer.deserialize_map(CustomPropertiesVisitor)
    }
}

#[derive(Debug, Clone, Serialize)]
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
                        _ => panic!("Unknown key for ClassInstance: {}", key),
                    }
                }

                Ok(ClassInstance {
                    name: name.unwrap(),
                    ty: ty.unwrap(),
                    properties: properties.unwrap(),
                })
            }
        }

        deserializer.deserialize_map(ClassInstanceVisitor)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertiesWrapper {
    #[serde(rename = "property")]
    pub properties: Vec<PropertyInstance>,
}

#[derive(Debug, Clone, Serialize)]
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
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "@name" => {
                            name = Some(map.next_value::<String>()?);
                        }
                        "@type" => {
                            ty = map.next_value::<String>()?;
                            if ty == "class" {
                                panic!("Nested class properties are not supported yet!");
                            }
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
                            _ => unreachable!(),
                        },
                        "property" => {
                            return map.next_value::<PropertyInstance>();
                        }
                        _ => panic!("Unknown key for PropertyInstance: {}", key),
                    }
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PropertyValue {
    Int(i32),
    Float(f32),
    Bool(bool),
    String(String),
    Color(TiledColor),
    Enum(String, String),
    ObjectRef(u32),
}
