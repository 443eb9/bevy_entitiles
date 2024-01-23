use std::fmt::Formatter;

use bevy::{math::Vec4, reflect::Reflect, render::color::Color};
use serde::{de::Visitor, Deserialize, Serialize};

use self::{
    default::*,
    layer::{ObjectLayer, TiledLayer},
};

pub mod default;
pub mod layer;
pub mod property;
pub mod tileset;

#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct TiledTilemap {
    /// The TMX format version. Was “1.0” so far,
    /// and will be incremented to match minor
    /// Tiled releases.
    #[serde(rename = "@version")]
    pub version: String,

    ///  The Tiled version used to save the file
    /// (since Tiled 1.0.1). May be a date (for
    /// snapshot builds). (optional)
    #[serde(rename = "@tiledversion")]
    pub tiled_version: String,

    ///  Map orientation. Tiled supports “orthogonal”,
    /// “isometric”, “staggered” and “hexagonal”
    /// (since 0.11).
    #[serde(rename = "@orientation")]
    pub orientation: MapOrientation,

    /// The order in which tiles on tile layers
    /// are rendered. Valid values are `right-down`
    /// (the default), `right-up`, `left-down` and
    /// `left-up`. In all cases, the map is drawn
    /// row-by-row. (only supported for orthogonal
    /// maps at the moment)
    #[serde(rename = "@renderorder")]
    pub render_order: TileRenderOrder,

    /// The map width in tiles.
    #[serde(rename = "@width")]
    pub width: u32,

    /// The map height in tiles.
    #[serde(rename = "@height")]
    pub height: u32,

    /// The width of a tile.
    #[serde(rename = "@tilewidth")]
    pub tile_width: u32,

    /// The height of a tile.
    #[serde(rename = "@tileheight")]
    pub tile_height: u32,

    /// Only for hexagonal maps. Determines the
    /// width or height (depending on the staggered
    /// axis) of the tile’s edge, in pixels.
    #[serde(rename = "@hexsidelength")]
    #[serde(default)]
    pub hex_side_length: u32,

    /// For staggered and hexagonal maps, determines
    /// which axis (“x” or “y”) is staggered. (since
    /// 0.11)
    #[serde(rename = "@staggeraxis")]
    #[serde(default)]
    pub stagger_axis: StaggeredAxis,

    /// For staggered and hexagonal maps, determines
    /// whether the “even” or “odd” indexes along
    /// the staggered axis are shifted. (since 0.11)
    #[serde(rename = "@staggerindex")]
    #[serde(default)]
    pub stagger_index: StaggeredIndex,

    /// X coordinate of the parallax origin in
    ///  pixels (defaults to 0). (since 1.8)
    #[serde(rename = "@parallaxoriginx")]
    #[serde(default)]
    pub parallax_origin_x: f32,

    /// Y coordinate of the parallax origin in
    ///  pixels (defaults to 0). (since 1.8)
    #[serde(rename = "@parallaxoriginy")]
    #[serde(default)]
    pub parallax_origin_y: f32,

    /// The background color of the map. (optional,
    /// may include alpha value since 0.15 in the
    /// form #AARRGGBB. Defaults to fully transparent.)
    #[serde(rename = "@backgroundcolor")]
    #[serde(default)]
    pub background_color: TiledColor,

    #[serde(rename = "tileset")]
    pub tilesets: Vec<TilesetDef>,

    #[serde(rename = "$value")]
    #[serde(default)]
    pub layers: Vec<TiledLayer>,

    #[serde(rename = "group")]
    #[serde(default)]
    pub groups: Vec<TiledGroup>,
}

#[derive(Debug, Clone, Reflect, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MapOrientation {
    Orthogonal,
    Isometric,
    Staggered,
    Hexagonal,
}

#[derive(Debug, Clone, Reflect, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum TileRenderOrder {
    RightDown,
    RightUp,
    LeftDown,
    LeftUp,
}

#[derive(Debug, Default, Clone, Reflect, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StaggeredAxis {
    X,
    #[default]
    Y,
}

#[derive(Debug, Default, Clone, Reflect, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StaggeredIndex {
    #[default]
    Odd,
    Even,
}

#[derive(Debug, Default, Clone, Reflect, Copy, Serialize)]
pub struct TiledColor {
    pub a: f32,
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl From<String> for TiledColor {
    fn from(value: String) -> Self {
        if value.len() == 7 {
            let r = u8::from_str_radix(&value[1..3], 16).unwrap() as f32 / 255.;
            let g = u8::from_str_radix(&value[3..5], 16).unwrap() as f32 / 255.;
            let b = u8::from_str_radix(&value[5..7], 16).unwrap() as f32 / 255.;
            Self { a: 1., r, g, b }
        } else {
            let a = u8::from_str_radix(&value[1..3], 16).unwrap() as f32 / 255.;
            let r = u8::from_str_radix(&value[3..5], 16).unwrap() as f32 / 255.;
            let g = u8::from_str_radix(&value[5..7], 16).unwrap() as f32 / 255.;
            let b = u8::from_str_radix(&value[7..9], 16).unwrap() as f32 / 255.;
            Self { a, r, g, b }
        }
    }
}

impl Into<Color> for TiledColor {
    fn into(self) -> Color {
        Color::rgba(self.r, self.g, self.b, self.a)
    }
}

impl Into<Vec4> for TiledColor {
    fn into(self) -> Vec4 {
        Vec4::new(self.r, self.g, self.b, self.a)
    }
}

impl<'de> Deserialize<'de> for TiledColor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        pub struct TiledColorVisitor;
        impl<'de> Visitor<'de> for TiledColorVisitor {
            type Value = TiledColor;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("a color in the format #AARRGGBB")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(TiledColor::from(v.to_string()))
            }
        }

        deserializer.deserialize_str(TiledColorVisitor)
    }
}

#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct TilesetDef {
    /// The first global tile ID of this tileset
    /// (this global ID maps to the first tile
    /// in this tileset).
    #[serde(rename = "@firstgid")]
    pub first_gid: u32,

    /// If this tileset is stored in an external
    /// TSX (Tile Set XML) file, this attribute
    /// refers to that file. That TSX file has the
    ///  same structure as the <tileset> element
    /// described here. (There is the firstgid
    /// attribute missing and this source attribute
    ///  is also not there. These two attributes
    /// are kept in the TMX map, since they are
    /// map specific.)
    #[serde(rename = "@source")]
    pub source: String,
}

#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub enum GroupContent {
    #[serde(rename = "layer")]
    Layer(TiledLayer),
    #[serde(rename = "objectgroup")]
    Group(ObjectLayer),
}

#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct TiledGroup {
    /// Unique ID of the layer (defaults to 0, with valid
    /// IDs being at least 1). Each layer that added to a
    /// map gets a unique id. Even if a layer is deleted,
    /// no layer ever gets the same ID. Can not be
    /// changed in Tiled. (since Tiled 1.2)
    #[serde(rename = "@id")]
    pub id: u32,

    /// The name of the layer. (defaults to “”)
    #[serde(rename = "@name")]
    pub name: String,

    /// The x coordinate of the layer in tiles.
    /// Defaults to 0 and can not be changed in Tiled.
    #[serde(rename = "@x")]
    #[serde(default)]
    pub x: i32,

    /// The y coordinate of the layer in tiles.
    /// Defaults to 0 and can not be changed in Tiled.
    #[serde(rename = "@y")]
    #[serde(default)]
    pub y: i32,

    /// The opacity of the layer as a value from 0 to
    /// 1. Defaults to 1.
    #[serde(rename = "@opacity")]
    #[serde(default = "default_onef")]
    pub opacity: f32,

    /// Whether the layer is shown (1) or hidden (0).
    /// Defaults to 1.
    #[serde(rename = "@visible")]
    #[serde(default = "default_true")]
    pub visible: bool,

    /// A tint color that is multiplied with any
    /// tiles drawn by this layer in #AARRGGBB or
    /// #RRGGBB format (optional).
    #[serde(rename = "@tintcolor")]
    #[serde(default = "default_white")]
    pub tint: TiledColor,

    /// Horizontal offset for this layer in pixels.
    /// Defaults to 0. (since 0.14)
    #[serde(rename = "@offsetx")]
    #[serde(default)]
    pub offset_x: f32,

    /// Vertical offset for this layer in pixels.
    /// Defaults to 0. (since 0.14)
    #[serde(rename = "@offsety")]
    #[serde(default)]
    pub offset_y: f32,

    /// Horizontal parallax factor for this layer.
    /// Defaults to 1. (since 1.5)
    #[serde(rename = "@parallaxx")]
    #[serde(default = "default_onef")]
    pub parallax_x: f32,

    /// Vertical parallax factor for this layer.
    /// Defaults to 1. (since 1.5)
    #[serde(rename = "@parallaxy")]
    #[serde(default = "default_onef")]
    pub parallax_y: f32,

    /// The width of the layer in tiles. Always
    /// the same as the map width for fixed-size maps.
    #[serde(rename = "@width")]
    #[serde(default)]
    pub width: u32,

    /// The height of the layer in tiles. Always
    /// the same as the map height for fixed-size maps.
    #[serde(rename = "@height")]
    #[serde(default)]
    pub height: u32,

    #[serde(rename = "$value")]
    pub content: Vec<GroupContent>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse() {
        let map = quick_xml::de::from_str::<TiledTilemap>(
            std::fs::read_to_string("assets/tiled/tilemaps/isometric.tmx")
                .unwrap()
                .as_str(),
        )
        .unwrap();

        dbg!(map);
    }
}
