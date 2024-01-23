use bevy::reflect::Reflect;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
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
