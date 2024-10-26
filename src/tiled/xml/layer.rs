use std::fmt::Formatter;

use bevy::{
    ecs::system::EntityCommands,
    math::{IVec2, Vec2},
    prelude::{Component, Deref, DerefMut},
    reflect::Reflect,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use serde::{
    de::{IgnoredAny, Visitor},
    Deserialize, Serialize,
};

use crate::{
    tiled::{
        resources::{PackedTiledTilemap, TiledAssets, TiledCustomTileInstance},
        xml::{default::*, property::Components, MapOrientation, TiledColor},
    },
    tilemap::{
        coordinates,
        map::TileRenderSize,
        tile::{TileBuilder, TileFlip, TileLayer},
    },
};

#[cfg(feature = "physics")]
use avian2d::collision::Collider;

#[cfg(feature = "physics")]
use std::f32::consts::PI;

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
        tiled_data: &'a PackedTiledTilemap,
    ) -> impl Iterator<
        Item = (
            IVec2,
            TileBuilder,
            TileRenderSize,
            Option<&TiledCustomTileInstance>,
        ),
    > + 'a {
        self.0
            .iter()
            .enumerate()
            .filter_map(move |(index, tile_id)| {
                if *tile_id == 0 {
                    return None;
                }

                let mut builder = TileBuilder::new();
                let mut layer = TileLayer::default();

                let flip = *tile_id >> 30;
                let tile_id = tile_id & 0x3FFF_FFFF;

                let (tileset, tileset_meta) = tiled_assets.get_tileset(tile_id);
                let atlas_index = tile_id - tileset_meta.first_gid;

                layer.flip = TileFlip::from_bits(flip).unwrap();
                layer.texture_index = tileset_meta.texture_index as i32;

                if let Some(anim) = tileset.animated_tiles.get(&atlas_index) {
                    builder = builder.with_animation(anim.clone());
                } else {
                    layer.atlas_index = atlas_index as i32;
                    builder = builder.with_layer(0, layer);
                }

                let mut index = IVec2::new(index as i32 % size.x, index as i32 / size.x);

                match tiled_data.xml.orientation {
                    MapOrientation::Orthogonal => {}
                    MapOrientation::Isometric => index = IVec2::new(index.y, index.x),
                    MapOrientation::Staggered | MapOrientation::Hexagonal => {
                        index = coordinates::destaggerize_index(
                            index,
                            tiled_data.xml.stagger_index.into(),
                        );
                    }
                }

                Some((
                    index,
                    builder,
                    TileRenderSize(tileset.texture.desc.tile_size.as_vec2()),
                    tileset.custom_properties_tiles.get(&atlas_index),
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
    pub objects: Vec<TiledObjectInstance>,
}

#[derive(Debug, Clone, Reflect, Serialize)]
pub struct TiledObjectInstance {
    /// Unique ID of the object (defaults to 0,
    /// with valid IDs being at least 1). Each
    /// object that is placed on a map gets a
    /// unique id. Even if an object was deleted,
    /// no object gets the same ID. Can not be
    /// changed in Tiled. (since Tiled 0.11)
    pub id: u32,

    /// The name of the object. An arbitrary
    /// string. (defaults to “”)
    pub name: String,

    /// The class of the object. An arbitrary
    /// string. (defaults to “”, was saved as
    /// class in 1.9)
    pub ty: String,

    /// The x coordinate of the object in pixels.
    /// (defaults to 0)
    pub x: f32,

    /// The y coordinate of the object in pixels.
    /// (defaults to 0)
    pub y: f32,

    /// The width of the object in pixels.
    /// (defaults to 0)
    pub width: f32,

    /// The height of the object in pixels.
    /// (defaults to 0)
    pub height: f32,

    /// The rotation of the object in degrees
    /// clockwise around (x, y). (defaults to 0)
    pub rotation: f32,

    /// A reference to a tile. (optional)
    pub gid: Option<u32>,

    /// Whether the object is shown (1) or hidden
    /// (0). (defaults to 1)
    pub visible: bool,

    pub properties: Components,

    pub shape: ObjectShape,
}

impl<'de> Deserialize<'de> for TiledObjectInstance {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct TiledObjectInstanceVisitor;
        impl<'de> Visitor<'de> for TiledObjectInstanceVisitor {
            type Value = TiledObjectInstance;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string or a sequence")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut id = None;
                let mut name = None;
                let mut ty = None;
                let mut x = None;
                let mut y = None;
                let mut width = None;
                let mut height = None;
                let mut rotation = None;
                let mut gid = None;
                let mut visible = None;
                let mut properties = None;
                let mut shape = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "@id" => id = Some(map.next_value::<u32>()?),
                        "@name" => name = Some(map.next_value::<String>()?),
                        "@type" => ty = Some(map.next_value::<String>()?),
                        "@x" => x = Some(map.next_value::<f32>()?),
                        "@y" => y = Some(map.next_value::<f32>()?),
                        "@width" => width = Some(map.next_value::<f32>()?),
                        "@height" => height = Some(map.next_value::<f32>()?),
                        "@rotation" => rotation = Some(map.next_value::<f32>()?),
                        "@gid" => gid = Some(map.next_value::<u32>()?),
                        "@visible" => visible = Some(map.next_value::<bool>()?),
                        "properties" => properties = Some(map.next_value::<Components>()?),
                        "ellipse" => {
                            map.next_value::<IgnoredAny>()?;
                            shape = Some(ObjectShape::Ellipse);
                        }
                        "polygon" => shape = Some(ObjectShape::Polygon(map.next_value()?)),
                        "point" => {
                            map.next_value::<IgnoredAny>()?;
                            shape = Some(ObjectShape::Point);
                        }
                        _ => panic!("Unknown key for TiledObjectInstance: {}", key),
                    }
                }

                Ok(TiledObjectInstance {
                    id: id.unwrap(),
                    name: name.unwrap_or_default(),
                    ty: ty.unwrap_or_default(),
                    x: x.unwrap_or_default(),
                    y: y.unwrap_or_default(),
                    width: width.unwrap_or_default(),
                    height: height.unwrap_or_default(),
                    rotation: rotation.unwrap_or_default(),
                    gid,
                    visible: visible.unwrap_or(true),
                    properties: properties.unwrap_or_default(),
                    shape: shape.unwrap_or_default(),
                })
            }
        }

        deserializer.deserialize_map(TiledObjectInstanceVisitor)
    }
}

impl TiledObjectInstance {
    pub fn spawn_sprite(&self, commands: &mut EntityCommands, tiled_assets: &TiledAssets) {
        if self.visible {
            commands.insert(MaterialMesh2dBundle {
                material: tiled_assets.clone_object_material_handle(self.id),
                mesh: Mesh2dHandle(tiled_assets.clone_object_mesh_handle(self.id)),
                ..Default::default()
            });
        }
    }

    #[cfg(not(feature = "physics"))]
    pub fn instantiate_shape(&self, commands: &mut EntityCommands) {
        if !matches!(self.shape, ObjectShape::Point) {
            bevy::log::error!("To spawn colliders, please enable `physics` feature first.");
            return;
        }

        commands.insert(match self.shape {
            ObjectShape::Point => TiledPointObject(Vec2 {
                x: self.x,
                y: self.y,
            }),
            ObjectShape::Ellipse | ObjectShape::Polygon(_) | ObjectShape::Rect => unreachable!(),
        });
    }

    #[cfg(feature = "physics")]
    pub fn instantiate_shape(&self, commands: &mut EntityCommands) {
        match &self.shape {
            ObjectShape::Point => {
                commands.insert(TiledPointObject(Vec2 {
                    x: self.x,
                    y: self.y,
                }));
            }
            ObjectShape::Ellipse => {
                commands.insert(Collider::ellipse(self.width / 2., self.height / 2.));
            }
            ObjectShape::Polygon(polygon) => {
                let mut points = polygon.points.clone();
                points.push(polygon.points[0]);
                commands.insert(Collider::polyline(
                    points
                        .into_iter()
                        .map(|v| {
                            Vec2::from_angle(-self.rotation / 180. * PI)
                                .rotate(Vec2::new(v.x, -v.y))
                        })
                        .collect(),
                    None,
                ));
            }
            ObjectShape::Rect => {
                commands.insert(if self.gid.is_none() {
                    Collider::rectangle(self.width, self.height)
                } else {
                    Collider::convex_hull(
                        [
                            Vec2::ZERO,
                            Vec2::new(self.width, 0.),
                            Vec2::new(self.width, self.height),
                            Vec2::new(0., self.height),
                        ]
                        .into_iter()
                        .map(|v| v + Vec2::new(-self.width / 2., self.height / 2.))
                        .collect(),
                    )
                    .unwrap()
                });
            }
        };
    }
}

#[derive(Component, Default, Clone, Copy, Deref, DerefMut)]
pub struct TiledPointObject(pub Vec2);

#[derive(Debug, Default, Clone, Reflect, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ObjectShape {
    Point,
    Ellipse,
    Polygon(Polygon),
    #[default]
    Rect,
}

#[derive(Debug, Clone, Reflect, Serialize)]
pub struct Polygon {
    pub points: Vec<Vec2>,
}

impl<'de> Deserialize<'de> for Polygon {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct PolygonVisitor;
        impl<'de> Visitor<'de> for PolygonVisitor {
            type Value = Polygon;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string in format `x1,y1 x2,y2 x3,y3 ...`")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut points = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "@points" => points = Some(map.next_value::<String>()?),
                        _ => panic!("Unknown key for Polygon: {}", key),
                    }
                }

                Ok(Polygon {
                    points: points
                        .unwrap()
                        .split(' ')
                        .into_iter()
                        .map(|p| {
                            let components =
                                p.split(',').map(|c| c.parse().unwrap()).collect::<Vec<_>>();
                            Vec2::new(components[0], components[1])
                        })
                        .collect(),
                })
            }
        }

        deserializer.deserialize_map(PolygonVisitor)
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_polygon() {
        let polygon = r#"
            <polygon points="0,0 0,32 -8,32 -8,48 16,48 16,32 8,32 8,0"/>
        "#;
        let polygon: Polygon = quick_xml::de::from_str(polygon).unwrap();
        assert_eq!(
            polygon.points,
            vec![
                Vec2::new(0., 0.),
                Vec2::new(0., 1.),
                Vec2::new(1., 1.),
                Vec2::new(1., 0.)
            ]
        );
    }
}
