use std::fmt::Formatter;

use bevy::{
    math::{IVec2, UVec2},
    reflect::Reflect,
};
use quick_xml::se;
use serde::{de::Visitor, Deserialize, Serialize};

use crate::{
    tiled::resources::TiledAssets,
    tilemap::{
        map::TilemapTexture,
        tile::{TileBuilder, TileLayer},
    },
};

use super::{default::*, property::Components, TiledColor};

#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub enum TiledLayer {
    #[serde(rename = "layer")]
    Tiles(ColorTileLayer),
    #[serde(rename = "objectgroup")]
    Objects(ObjectLayer),
    #[serde(rename = "imagelayer")]
    Image(ImageLayer),
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct ColorTileLayer {
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
    pub width: u32,

    /// The height of the layer in tiles. Always
    /// the same as the map height for fixed-size maps.
    #[serde(rename = "@height")]
    pub height: u32,

    pub data: ColorTileLayerData,
}

#[derive(Debug, Clone, Reflect, Serialize)]
#[serde(untagged)]
pub enum ColorTileLayerData {
    Tiles(TileData),
    Chunks(ChunkData),
}

impl<'de> Deserialize<'de> for ColorTileLayerData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ColorTileLayerDataVisitor;
        impl<'de> Visitor<'de> for ColorTileLayerDataVisitor {
            type Value = ColorTileLayerData;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string or a sequence")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut encoding = None;
                let mut compression = None;
                let mut chunks = vec![];
                let mut tiles = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "@encoding" => encoding = Some(map.next_value::<DataEncoding>()?),
                        "@compression" => compression = Some(map.next_value::<DataCompression>()?),
                        "chunk" => {
                            chunks.push(map.next_value::<Chunk>()?);
                        }
                        "$text" => {
                            tiles = Some(map.next_value::<Tiles>()?);
                        }
                        _ => panic!("Unknown key for ColorTileLayerData: {}", key),
                    }
                }

                if let Some(tiles) = tiles {
                    Ok(ColorTileLayerData::Tiles(TileData {
                        encoding: encoding.unwrap(),
                        compression: compression.unwrap_or_default(),
                        content: tiles,
                    }))
                } else {
                    Ok(ColorTileLayerData::Chunks(ChunkData {
                        encoding: encoding.unwrap(),
                        compression: compression.unwrap_or_default(),
                        content: chunks,
                    }))
                }
            }
        }

        deserializer.deserialize_map(ColorTileLayerDataVisitor)
    }
}

#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct TileData {
    /// The encoding used to encode the tile layer
    /// data. When used, it can be “base64” and
    /// “csv” at the moment. (optional)
    #[serde(rename = "@encoding")]
    pub encoding: DataEncoding,

    /// The compression used to compress the tile
    /// layer data. Tiled supports “gzip”, “zlib”
    /// and (as a compile-time option since Tiled
    /// 1.3) “zstd”.
    #[serde(rename = "@compression")]
    #[serde(default)]
    pub compression: DataCompression,

    #[serde(rename = "$value")]
    pub content: Tiles,
}

#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct ChunkData {
    /// The encoding used to encode the tile layer
    /// data. When used, it can be “base64” and
    /// “csv” at the moment. (optional)
    #[serde(rename = "@encoding")]
    pub encoding: DataEncoding,

    /// The compression used to compress the tile
    /// layer data. Tiled supports “gzip”, “zlib”
    /// and (as a compile-time option since Tiled
    /// 1.3) “zstd”.
    #[serde(rename = "@compression")]
    #[serde(default)]
    pub compression: DataCompression,

    #[serde(rename = "chunk")]
    pub content: Vec<Chunk>,
}

#[derive(Debug, Clone, Reflect, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DataEncoding {
    Csv,
    Base64,
}

#[derive(Debug, Default, Clone, Reflect, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DataCompression {
    #[default]
    None,
    Gzip,
    Zlib,
    Zstd,
}

#[derive(Debug, Clone, Reflect, Serialize)]
pub struct Tiles(pub Vec<u32>);

impl<'de> Deserialize<'de> for Tiles {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct TilesVisitor;
        impl<'de> Visitor<'de> for TilesVisitor {
            type Value = Tiles;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("a string or a sequence")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Tiles(
                    v.split(',')
                        .into_iter()
                        .map(|s| s.trim().parse::<u32>().unwrap())
                        .collect(),
                ))
            }
        }

        deserializer.deserialize_str(TilesVisitor)
    }
}

impl Tiles {
    pub fn iter_decoded<'a>(
        &'a self,
        size: IVec2,
        tiled_assets: &'a TiledAssets,
        tilemap_texture: &'a mut TilemapTexture,
        tilemap_name: &'a str,
    ) -> impl Iterator<Item = (IVec2, TileBuilder)> + 'a {
        let mut tileset = None;
        let mut first_gid = 0;
        self.0
            .iter()
            .enumerate()
            .filter_map(move |(index, texture)| {
                if *texture == 0 {
                    return None;
                }

                let texture = *texture;
                let tileset = tileset.unwrap_or_else(|| {
                    let (ts, gid) = tiled_assets.get_tileset(texture, tilemap_name);
                    tileset = Some(ts);
                    first_gid = gid;
                    *tilemap_texture = ts.texture.clone();
                    ts
                });

                let mut layer = TileLayer::new();
                if texture > i32::MAX as u32 {
                    let flip = texture >> 30;
                    layer = layer
                        .with_flip_raw(if flip == 3 { flip } else { flip ^ 3 })
                        .with_texture_index((texture & 0x3FFF_FFFF) - first_gid);
                } else {
                    layer = layer.with_texture_index(texture - first_gid);
                }

                assert!(
                    layer.texture_index < tileset.xml.tile_count as i32,
                    "Index {} is not in range [{}, {}]. Are you using \
                    multiple tilesets on one layer which is currently not supported?",
                    layer.texture_index,
                    0,
                    tileset.xml.tile_count - 1
                );

                Some((
                    IVec2::new(index as i32 % size.x, size.y - 1 - index as i32 / size.x),
                    TileBuilder::new().with_layer(0, layer),
                ))
            })
    }
}

#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct Chunk {
    /// The x coordinate of the chunk in tiles.
    #[serde(rename = "@x")]
    pub x: i32,

    /// The y coordinate of the chunk in tiles.
    #[serde(rename = "@y")]
    pub y: i32,

    /// The width of the chunk in tiles.
    #[serde(rename = "@width")]
    pub width: u32,

    /// The height of the chunk in tiles.
    #[serde(rename = "@height")]
    pub height: u32,

    #[serde(rename = "$value")]
    pub tiles: Tiles,
}

#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct ObjectLayer {
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

    #[serde(rename = "object")]
    pub objects: Vec<Object>,
}

#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct Object {
    /// Unique ID of the object (defaults to 0,
    /// with valid IDs being at least 1). Each
    /// object that is placed on a map gets a
    /// unique id. Even if an object was deleted,
    /// no object gets the same ID. Can not be
    /// changed in Tiled. (since Tiled 0.11)
    #[serde(rename = "@id")]
    pub id: u32,

    /// The name of the object. An arbitrary
    /// string. (defaults to “”)
    #[serde(rename = "@name")]
    #[serde(default)]
    pub name: String,

    /// The class of the object. An arbitrary
    /// string. (defaults to “”, was saved as
    /// class in 1.9)
    #[serde(rename = "@type")]
    #[serde(default)]
    pub ty: String,

    /// The x coordinate of the object in pixels.
    /// (defaults to 0)
    #[serde(rename = "@x")]
    pub x: f32,

    /// The y coordinate of the object in pixels.
    /// (defaults to 0)
    #[serde(rename = "@y")]
    pub y: f32,

    /// The width of the object in pixels.
    /// (defaults to 0)
    #[serde(rename = "@width")]
    #[serde(default)]
    pub width: f32,

    /// The height of the object in pixels.
    /// (defaults to 0)
    #[serde(rename = "@height")]
    #[serde(default)]
    pub height: f32,

    /// The rotation of the object in degrees
    /// clockwise around (x, y). (defaults to 0)
    #[serde(rename = "@rotation")]
    #[serde(default)]
    pub rotation: f32,

    /// A reference to a tile. (optional)
    #[serde(rename = "@gid")]
    pub gid: Option<u32>,

    /// Whether the object is shown (1) or hidden
    /// (0). (defaults to 1)
    #[serde(rename = "@visible")]
    #[serde(default = "default_true")]
    pub visible: bool,

    #[serde(default)]
    pub properties: Components,
}

#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct ImageLayer {
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

    /// Whether the image drawn by this layer is
    /// repeated along the X axis. (since Tiled 1.8)
    #[serde(rename = "@repeatx")]
    #[serde(default)]
    pub repeat_x: bool,

    /// Whether the image drawn by this layer is
    /// repeated along the Y axis. (since Tiled 1.8)
    #[serde(rename = "@repeaty")]
    #[serde(default)]
    pub repeat_y: bool,

    #[serde(rename = "$value")]
    pub image: Image,
}

#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct Image {
    /// The reference to the tileset image file
    /// (Tiled supports most common image formats).
    /// Only used if the image is not embedded.
    #[serde(rename = "@source")]
    pub source: String,

    /// The image width in pixels (optional, used for
    /// tile index correction when the image changes)
    #[serde(rename = "@width")]
    pub width: u32,

    /// The image height in pixels (optional)
    #[serde(rename = "@height")]
    pub height: u32,
}
