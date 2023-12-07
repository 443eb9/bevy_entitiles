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
    /// Project background color
    pub bg_color: LdtkColor,

    /// A structure containing all the definitions of this project
    pub defs: Definitions,

    /// If TRUE, one file will be saved for the project (incl. all its definitions)
    /// and one file in a sub-folder for each level.
    pub external_levels: bool,

    ///	Unique project identifier
    pub iid: String,

    /// File format version
    pub json_version: String,

    /// All levels. The order of this array is only relevant in
    /// `LinearHorizontal` and `linearVertical` world layouts (see `worldLayout` value).
    ///
    /// Otherwise, you should refer to the `worldX`,`worldY` coordinates of each Level.
    pub levels: Vec<Level>,

    /// All instances of entities that have their `exportToToc`flag enabled
    /// are listed in this array.
    pub toc: Vec<Toc>,

    /// ## WARNING:
    /// this field will move to the `worlds` array after the "multi-worlds" update.
    /// It will then be `null`. You can enable the Multi-worlds
    /// advanced project option to enable the change immediately.
    ///
    /// Height of the world grid in pixels.
    pub world_grid_height: Nullable<i32>,

    /// ## WARNING:
    /// this field will move to the `worlds` array after the "multi-worlds" update.
    /// It will then be `null`. You can enable the Multi-worlds
    /// advanced project option to enable the change immediately.
    ///
    /// Width of the world grid in pixels.
    pub world_grid_width: Nullable<i32>,

    /// ## WARNING:
    /// this field will move to the `worlds` array after the "multi-worlds" update.
    /// It will then be `null`. You can enable the Multi-worlds
    /// advanced project option to enable the change immediately.
    ///
    /// An enum that describes how levels are organized in this project (ie. linearly or in a 2D space).
    pub world_layout: Nullable<WorldLayout>,

    /// This array will be empty, unless you enable the Multi-Worlds in the project advanced settings.
    /// - in current version, a LDtk project file can only contain a single world with
    /// multiple levels in it. In this case, levels and world layout related settings
    /// are stored in the root of the JSON.
    /// - with "Multi-worlds" enabled, there will be a `worlds` array in root, each world
    /// containing levels and layout settings. Basically, it's pretty much only about
    /// moving the `levels` array to the `worlds` array, along with world layout related values
    /// (eg. `worldGridWidth` etc).
    ///
    /// If you want to start supporting this future update easily,
    /// please refer to this documentation: https://github.com/deepnight/ldtk/issues/231
    pub worlds: Vec<World>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Toc {
    ///
    pub identifier: String,

    ///
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
    ///
    pub world_grid_width: i32,

    ///
    pub iid: String,

    ///
    pub world_grid_height: i32,

    ///
    pub world_layout: Nullable<WorldLayout>,

    ///
    pub default_level_width: i32,

    ///
    pub levels: Vec<Level>,

    ///
    pub default_level_height: i32,

    ///
    pub identifier: String,
}
