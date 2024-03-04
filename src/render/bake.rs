use bevy::{
    asset::Assets,
    ecs::{
        component::Component,
        entity::Entity,
        query::With,
        system::{Commands, Query, Res},
    },
    log::warn,
    math::{primitives::Rectangle, IVec2, UVec2, Vec3, Vec4},
    reflect::Reflect,
    render::{
        mesh::Mesh,
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDescriptor, TextureDimension, TextureFormat},
        texture::{BevyDefault, Image},
    },
};

use crate::{
    math::aabb::IAabb2d,
    tilemap::{
        map::{
            TileRenderSize, TilemapLayerOpacities, TilemapSlotSize, TilemapStorage, TilemapTexture,
            TilemapType,
        },
        tile::{Tile, TileLayer, TileTexture},
    },
    MAX_LAYER_COUNT,
};

/// A component that marks an tilemap entity to be baked into a **static** quad mesh.
#[derive(Component, Reflect)]
pub struct TilemapBaker {
    /// If true, the baked tilemap will be saved as `BakedTilemap` and add back to the tilemap entity.
    ///
    /// If `remove_after_done` is also true, the component will be added to a new entity.
    pub into_data: bool,
    /// If true, the tilemap entity will be removed after the baking is done.
    pub remove_after_done: bool,
}

#[derive(Component, Reflect)]
pub struct BakedTilemap {
    pub size_px: UVec2,
    /// Ignore the `Option`, it's just used for taking the `Image` out without cloning.
    /// You can always unwrap this.
    pub texture: Option<Image>,
}

pub fn tilemap_baker(
    mut commands: Commands,
    mut tilemaps_query: Query<(
        Entity,
        &TilemapSlotSize,
        &TilemapType,
        &mut TilemapStorage,
        &TilemapLayerOpacities,
        &TilemapTexture,
        &TilemapBaker,
    )>,
    tiles_query: Query<&Tile>,
    image_assets: Res<Assets<Image>>,
) {
    for (tilemap_entity, slot_size, tilemap_ty, mut storage, opacities, texture, baker) in
        &mut tilemaps_query
    {
        let chunk_size = storage.storage.chunk_size as i32;
        let mut tilemap_aabb = IAabb2d::default();

        let tiles = storage
            .storage
            .chunks
            .iter()
            .flat_map(|(ci, c)| {
                c.iter().enumerate().filter_map(move |(ti, t)| {
                    if let Some(tile) = t {
                        Some((
                            *ci * chunk_size
                                + IVec2 {
                                    x: ti as i32 % chunk_size,
                                    y: ti as i32 / chunk_size,
                                },
                            *tile,
                        ))
                    } else {
                        None
                    }
                })
            })
            .filter_map(|(tile_index, tile_entity)| {
                let Ok(tile) = tiles_query.get(tile_entity) else {
                    return None;
                };

                tilemap_aabb.expand_to_contain(tile_index);
                Some((tile_index, tile))
            })
            .collect::<Vec<_>>();

        let texture_image = image_assets.get(texture.handle()).unwrap();
        let target_size = tilemap_aabb.size().as_uvec2() * texture.desc.tile_size;
        let mut bake_target = vec![0; (target_size.x * target_size.y * 4) as usize];

        tiles.into_iter().for_each(|(tile_index, tile)| {
            let mut rel_index = (tile_index - tilemap_aabb.min).as_uvec2();
            rel_index.y = tilemap_aabb.size().y as u32 - rel_index.y - 1;

            match &tile.texture {
                TileTexture::Static(layers) => layers
                    .iter()
                    .rev()
                    .take(MAX_LAYER_COUNT)
                    .enumerate()
                    .filter_map(|(i, l)| {
                        if l.texture_index >= 0 {
                            Some((opacities.0[MAX_LAYER_COUNT - i - 1], l))
                        } else {
                            None
                        }
                    })
                    .rev()
                    .for_each(|(opacity, layer)| {
                        set_tile(
                            texture,
                            texture_image,
                            rel_index,
                            target_size,
                            &mut bake_target,
                            layer,
                            opacity,
                        );
                    }),
                TileTexture::Animated(_) => {
                    warn!("Skipping animated tile at {:?}", tile_index);
                }
            };
        });

        let baked_tilemap = BakedTilemap {
            size_px: target_size,
            texture: Some(Image::new(
                Extent3d {
                    width: target_size.x,
                    height: target_size.y,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                bake_target,
                TextureFormat::bevy_default(),
                RenderAssetUsages::all(),
            )),
        };

        commands.entity(tilemap_entity).remove::<TilemapBaker>();

        if baker.into_data {
            if baker.remove_after_done {
                commands.spawn(baked_tilemap);
            } else {
                commands.entity(tilemap_entity).insert(baked_tilemap);
            }
        }

        if baker.remove_after_done {
            storage.despawn(&mut commands);
        }
    }
}

fn set_tile(
    texture: &TilemapTexture,
    texture_image: &Image,
    rel_index: UVec2,
    target_size: UVec2,
    bake_target: &mut Vec<u8>,
    layer: &TileLayer,
    opacity: f32,
) {
    let tile_px = texture.get_atlas_urect(layer.texture_index as u32);
    let tile_size = tile_px.size();

    for mut y in 0..texture.desc.tile_size.y {
        for mut x in 0..texture.desc.tile_size.x {
            let tile_px_col = get_pixel(
                &texture_image.data,
                texture.desc.size,
                tile_px.min + UVec2 { x, y },
            );

            let map_px_col = get_pixel(
                bake_target,
                target_size,
                rel_index * texture.desc.tile_size + UVec2 { x, y },
            );

            let final_px_col = alpha_blend(map_px_col, tile_px_col, opacity);

            set_pixel(
                bake_target,
                target_size,
                rel_index * texture.desc.tile_size + UVec2 { x, y },
                final_px_col,
            );
        }
    }
}

fn set_pixel(buffer: &mut Vec<u8>, mut image_size: UVec2, pos: UVec2, value: [u8; 4]) {
    image_size.x *= 4;
    let index = (pos.y * image_size.x + pos.x * 4) as usize;
    buffer[index..index + 4].copy_from_slice(&value);
}

fn get_pixel(buffer: &Vec<u8>, mut image_size: UVec2, pos: UVec2) -> [u8; 4] {
    image_size.x *= 4;
    let index = (pos.y * image_size.x + pos.x * 4) as usize;
    [
        buffer[index],
        buffer[index + 1],
        buffer[index + 2],
        buffer[index + 3],
    ]
}

fn alpha_blend(a: [u8; 4], b: [u8; 4], opacity: f32) -> [u8; 4] {
    let a = Vec4::new(a[0] as f32, a[1] as f32, a[2] as f32, a[3] as f32) / 255.;
    let b = Vec4::new(b[0] as f32, b[1] as f32, b[2] as f32, b[3] as f32) / 255.;

    let result = a.lerp(b, b.w * opacity);

    [
        (result.x * 255.) as u8,
        (result.y * 255.) as u8,
        (result.z * 255.) as u8,
        (result.w * 255.) as u8,
    ]
}
