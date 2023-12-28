use bevy::{math::Vec4, reflect::Reflect, render::color::Color};
use serde::{de::Visitor, Deserialize, Serialize};

use self::{definitions::Definitions, level::Level};

pub mod definitions;
pub mod field;
pub mod level;
pub mod macros;

#[derive(Serialize, Debug, Clone, Copy, Reflect)]
pub struct LdtkColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl From<String> for LdtkColor {
    fn from(value: String) -> Self {
        let r = u8::from_str_radix(&value[1..3], 16).unwrap() as f32 / 255.;
        let g = u8::from_str_radix(&value[3..5], 16).unwrap() as f32 / 255.;
        let b = u8::from_str_radix(&value[5..7], 16).unwrap() as f32 / 255.;
        Self { r, g, b }
    }
}

impl Into<Color> for LdtkColor {
    fn into(self) -> Color {
        Color::rgb(self.r, self.g, self.b)
    }
}

impl Into<Vec4> for LdtkColor {
    fn into(self) -> Vec4 {
        Vec4::new(self.r, self.g, self.b, 1.)
    }
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
        Ok(LdtkColor::from(value.to_string()))
    }
}

#[derive(Serialize, Deserialize, Debug, Reflect)]
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
    pub world_grid_height: Option<i32>,

    /// ## WARNING:
    /// this field will move to the `worlds` array after the "multi-worlds" update.
    /// It will then be `null`. You can enable the Multi-worlds
    /// advanced project option to enable the change immediately.
    ///
    /// Width of the world grid in pixels.
    pub world_grid_width: Option<i32>,

    /// ## WARNING:
    /// this field will move to the `worlds` array after the "multi-worlds" update.
    /// It will then be `null`. You can enable the Multi-worlds
    /// advanced project option to enable the change immediately.
    ///
    /// An enum that describes how levels are organized in this project (ie. linearly or in a 2D space).
    pub world_layout: Option<WorldLayout>,

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

#[derive(Serialize, Deserialize, Debug, Reflect)]
#[serde(rename_all = "camelCase")]
pub struct Toc {
    pub identifier: String,
    pub instances: Vec<EntityRef>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Reflect)]
pub enum WorldLayout {
    Free,
    GridVania,
    LinearHorizontal,
    LinearVertical,
}

#[derive(Serialize, Deserialize, Debug, Reflect)]
#[serde(rename_all = "camelCase")]
pub struct World {
    /// Width of the world grid in pixels.
    pub world_grid_width: i32,

    /// Unique instance identifer
    pub iid: String,

    /// Height of the world grid in pixels.
    pub world_grid_height: i32,

    /// An enum that describes how levels are organized in this project
    /// (ie. linearly or in a 2D space).
    /// Possible values: `Free`, `GridVania`, `LinearHorizontal`, `LinearVertical`
    pub world_layout: Option<WorldLayout>,

    /// All levels from this world.
    /// The order of this array is only relevant in `LinearHorizontal` and
    /// `linearVertical` world layouts (see `worldLayout` value). Otherwise,
    /// you should refer to the `worldX`,`worldY` coordinates of each Level.
    pub levels: Vec<Level>,

    /// User defined unique identifier
    pub identifier: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect)]
#[serde(rename_all = "camelCase")]
pub struct EntityRef {
    /// IID of the refered EntityInstance
    pub entity_iid: String,

    /// IID of the LayerInstance containing the refered EntityInstance
    pub layer_iid: String,

    /// IID of the Level containing the refered EntityInstance
    pub level_iid: String,

    /// IID of the World containing the refered EntityInstance
    pub world_iid: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect)]
#[serde(rename_all = "camelCase")]
pub struct GridPoint {
    /// X grid-based coordinate
    pub cx: i32,

    /// Y grid-based coordinate
    pub cy: i32,
}
