use bevy::{asset::Asset, reflect::Reflect};
use serde::{Deserialize, Serialize};

use crate::tiled::xml::property::Components;

#[derive(Asset, Debug, Clone, Reflect, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct TiledTileset {
    /// The name of this tileset.
    #[serde(rename = "@name")]
    pub name: String,

    /// The (maximum) width of the tiles in this
    /// tileset. Irrelevant for image collection
    /// tilesets, but stores the maximum tile width.
    #[serde(rename = "@tilewidth")]
    pub tile_width: u32,

    /// The (maximum) height of the tiles in this
    /// tileset. Irrelevant for image collection
    /// tilesets, but stores the maximum tile height.
    #[serde(rename = "@tileheight")]
    pub tile_height: u32,

    /// The spacing in pixels between the tiles
    /// in this tileset (applies to the tileset
    /// image, defaults to 0). Irrelevant for
    /// image collection tilesets.
    #[serde(rename = "@spacing")]
    #[serde(default)]
    pub spacing: u32,

    /// The margin around the tiles in this tileset
    /// (applies to the tileset image, defaults to
    /// 0). Irrelevant for image collection tilesets.
    #[serde(rename = "@margin")]
    #[serde(default)]
    pub margin: u32,

    /// The number of tiles in this tileset (since
    /// 0.13). Note that there can be tiles with a
    /// higher ID than the tile count, in case the
    /// tileset is an image collection from which
    /// tiles have been removed.
    #[serde(rename = "@tilecount")]
    pub tile_count: u32,

    /// The number of tile columns in the tileset.
    /// For image collection tilesets it is editable
    /// and is used when displaying the tileset.
    /// (since 0.15)
    #[serde(rename = "@columns")]
    pub columns: u32,

    /// Controls the alignment for tile objects.
    /// The default value is `unspecified`, for
    /// compatibility reasons. When `unspecified`,
    /// tile objects use `bottomleft` in orthogonal
    /// mode and `bottom` in isometric mode.
    #[serde(rename = "@objectalignment")]
    #[serde(default)]
    pub object_alignment: ObjectAlignment,

    /// The fill mode to use when rendering tiles
    /// from this tileset. Only relevant when the
    /// tiles are not rendered at their native size,
    /// so this applies to resized tile objects
    /// or in combination with `tilerendersize` set
    /// to `grid`. (since 1.9)
    #[serde(rename = "@fillmode")]
    #[serde(default)]
    pub fill_mode: FillMode,

    pub image: TilesetImage,

    #[serde(default)]
    pub transformations: TilesetTransformations,

    /// Generally, animated tiles.
    #[serde(rename = "tile")]
    #[serde(default)]
    pub special_tiles: Vec<TiledTile>,
}

#[derive(Debug, Default, Clone, Reflect, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ObjectAlignment {
    #[default]
    Unspecified,
    TopLeft,
    Top,
    TopRight,
    Left,
    Center,
    Right,
    BottomLeft,
    Bottom,
    BottomRight,
}

#[derive(Debug, Default, Clone, Reflect, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum FillMode {
    #[default]
    Stretch,
    PreserveAspectFit,
}

#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct TilesetImage {
    /// The reference to the tileset image file
    /// (Tiled supports most common image formats).
    /// Only used if the image is not embedded.
    #[serde(rename = "@source")]
    pub source: String,

    /// The image width in pixels (optional, used
    /// for tile index correction when the image
    /// changes)
    #[serde(rename = "@width")]
    pub width: u32,

    /// The image height in pixels (optional)
    #[serde(rename = "@height")]
    pub height: u32,
}

#[derive(Debug, Default, Clone, Reflect, Serialize, Deserialize)]
pub struct TilesetTransformations {
    /// Whether the tiles in this set can be
    /// flipped horizontally (default 0)
    #[serde(rename = "@hflip")]
    pub h_flip: bool,

    /// Whether the tiles in this set can be
    /// flipped vertically (default 0)
    #[serde(rename = "@vflip")]
    pub v_flip: bool,

    /// Whether the tiles in this set can be
    /// rotated in 90 degree increments (default 0)
    #[serde(rename = "@rotate")]
    pub rotate: u32,

    /// Whether untransformed tiles remain
    /// preferred, otherwise transformed tiles are
    /// used to produce more variations (default 0)
    #[serde(rename = "@preferuntransformed")]
    pub prefer_untransformed: bool,
}

#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct TiledTile {
    /// The local tile ID within its tileset.
    #[serde(rename = "@id")]
    pub id: u32,

    /// The class of the tile. Is inherited by tile objects.
    /// (since 1.0, defaults to “”, was saved as class in 1.9)
    #[serde(rename = "@type")]
    #[serde(default)]
    pub ty: String,

    /// A percentage indicating the probability that this
    /// tile is chosen when it competes with others while editing with the terrain tool. (defaults to 0)
    #[serde(rename = "@probability")]
    #[serde(default)]
    pub probability: f32,

    /// The X position of the sub-rectangle representing
    /// this tile (default: 0)
    #[serde(rename = "@x")]
    #[serde(default)]
    pub x: u32,

    /// The Y position of the sub-rectangle representing
    /// this tile (default: 0)
    #[serde(rename = "@y")]
    #[serde(default)]
    pub y: u32,

    /// The width of the sub-rectangle representing this
    /// tile (defaults to the image width)
    #[serde(rename = "@width")]
    #[serde(default)]
    pub width: u32,

    /// The height of the sub-rectangle representing this
    /// tile (defaults to the image height)
    #[serde(rename = "@height")]
    #[serde(default)]
    pub height: u32,

    #[serde(default)]
    pub animation: Option<TiledAnimation>,

    #[serde(default)]
    pub properties: Option<Components>,
}

#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct TiledAnimation {
    #[serde(rename = "frame")]
    pub frames: Vec<TiledAnimationFrame>,
}

#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct TiledAnimationFrame {
    /// The local ID of a tile within the parent <tileset>.
    #[serde(rename = "@tileid")]
    pub tile_id: u32,

    /// How long (in milliseconds) this frame should be
    /// displayed before advancing to the next frame.
    #[serde(rename = "@duration")]
    pub duration: u32,
}
