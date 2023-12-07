use bevy::math::Vec4;
use serde::{de::Visitor, Deserialize, Serialize};

use super::{
    definitions::Definitions,
    level::{EntityRef, Level},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Nullable<T> {
    Data(T),
    Null,
}

impl<T> Nullable<T> {
    pub fn unwrap(self) -> T {
        match self {
            Nullable::Data(data) => data,
            Nullable::Null => panic!("Tried to unwrap a null value"),
        }
    }
}

#[derive(Serialize, Debug)]
pub struct LdtkColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl<'de> Deserialize<'de> for LdtkColor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(LdtkColorVisitor)
    }
}

pub struct LdtkColorVisitor;

impl<'de> Visitor<'de> for LdtkColorVisitor {
    type Value = LdtkColor;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a color in the format #RRGGBB")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let r = u8::from_str_radix(&value[1..3], 16).unwrap() as f32 / 255.0;
        let g = u8::from_str_radix(&value[3..5], 16).unwrap() as f32 / 255.0;
        let b = u8::from_str_radix(&value[5..7], 16).unwrap() as f32 / 255.0;

        Ok(LdtkColor { r, g, b })
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LdtkJson {
    pub bg_color: LdtkColor,
    pub defs: Definitions,
    pub external_levels: bool,
    pub iid: String,
    pub json_version: String,
    pub levels: Vec<Level>,
    pub toc: Vec<Toc>,
    pub world_grid_height: Nullable<i32>,
    pub world_grid_width: Nullable<i32>,
    pub world_layout: Nullable<WorldLayout>,
    pub worlds: Vec<World>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Toc {
    pub identifier: String,
    pub instances: Vec<EntityRef>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum WorldLayout {
    Free,
    GridVanilla,
    LinearHorizontal,
    LinearVertical,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct World {
    pub world_grid_width: i32,
    pub iid: String,
    pub world_grid_height: i32,
    pub world_layout: Nullable<WorldLayout>,
    pub default_level_width: i32,
    pub levels: Vec<Level>,
    pub default_level_height: i32,
    pub identifier: String,
}
