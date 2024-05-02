use bevy::{
    asset::{Assets, Handle},
    ecs::{
        component::Component,
        entity::Entity,
        system::{Commands, Query, Res},
    },
    log::warn,
    math::{IVec2, UVec2, Vec2, Vec4, Vec4Swizzles},
    reflect::Reflect,
    render::{
        color::Color,
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
        texture::{BevyDefault, Image},
    },
};

use crate::{
    math::aabb::IAabb2d,
    tilemap::{
        map::{
            TileRenderSize, TilemapLayerOpacities, TilemapSlotSize, TilemapStorage, TilemapTexture,
            TilemapTextures,
        },
        tile::{Tile, TileFlip, TileLayer, TileTexture},
    },
    MAX_LAYER_COUNT,
};

/// A component that marks an tilemap entity to be baked into a **static** quad mesh.
#[derive(Component, Reflect)]
pub struct TilemapBaker {
    /// If true, the tilemap entity will be removed after the baking is done,
    /// and the baked tilemap will be spawned as a new entity.
    pub remove_after_done: bool,
}

#[derive(Component, Reflect)]
pub struct BakedTilemap {
    pub size_px: UVec2,
    pub slot_size: Vec2,
    pub tile_render_size: Vec2,
    /// Ignore the `Option`, it's just used for taking the `Image` out without cloning.
    /// You can always unwrap this.
    pub texture: Option<Image>,
}

pub fn tilemap_baker(
    mut commands: Commands,
    mut tilemaps_query: Query<(
        Entity,
        &TileRenderSize,
        &TilemapSlotSize,
        &mut TilemapStorage,
        &TilemapLayerOpacities,
        &Handle<TilemapTextures>,
        &TilemapBaker,
    )>,
    tiles_query: Query<&Tile>,
    image_assets: Res<Assets<Image>>,
    textures_assets: Res<Assets<TilemapTextures>>,
) {
    for (tilemap_entity, tile_render_size, slot_size, mut storage, opacities, texture, baker) in
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

        let textures = textures_assets.get(texture).unwrap();
        textures.assert_uniform_tile_size();
        let texture_images = textures
            .textures
            .iter()
            .map(|tex| image_assets.get(tex.handle()).unwrap())
            .collect::<Vec<_>>();
        let target_size = tilemap_aabb.size().as_uvec2() * textures.textures[0].desc.size;
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
                            Some((opacities.0[i], l))
                        } else {
                            None
                        }
                    })
                    .for_each(|(opacity, layer)| {
                        set_tile(
                            &textures.textures,
                            &texture_images,
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

            set_tile_tint(
                textures.textures[0].desc.tile_size,
                rel_index,
                target_size,
                &mut bake_target,
                tile.tint,
            );
        });

        let baked_tilemap = BakedTilemap {
            size_px: target_size,
            slot_size: slot_size.0,
            tile_render_size: tile_render_size.0,
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

        if baker.remove_after_done {
            storage.despawn(&mut commands);
            commands.spawn(baked_tilemap);
        } else {
            commands.entity(tilemap_entity).insert(baked_tilemap);
        }
    }
}

fn set_tile(
    textures: &[TilemapTexture],
    texture_images: &[&Image],
    rel_index: UVec2,
    target_size: UVec2,
    bake_target: &mut [u8],
    layer: &TileLayer,
    opacity: f32,
) {
    let texture = &textures[layer.texture_index as usize];
    let texture_image = &texture_images[layer.texture_index as usize];
    let tile_px = texture.get_atlas_urect(layer.atlas_index as u32);

    for mut y in 0..texture.desc.tile_size.y {
        for mut x in 0..texture.desc.tile_size.x {
            if layer.flip.contains(TileFlip::HORIZONTAL) {
                x = texture.desc.tile_size.x - x - 1;
            }
            if layer.flip.contains(TileFlip::VERTICAL) {
                y = texture.desc.tile_size.y - y - 1;
            }

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

fn set_tile_tint(
    tile_size: UVec2,
    rel_index: UVec2,
    target_size: UVec2,
    bake_target: &mut Vec<u8>,
    tint: Color,
) {
    let tint = tint.rgba_to_vec4();

    for y in 0..tile_size.y {
        for x in 0..tile_size.x {
            let map_px_col = get_pixel(
                bake_target,
                target_size,
                rel_index * tile_size + UVec2 { x, y },
            );

            set_pixel(
                bake_target,
                target_size,
                rel_index * tile_size + UVec2 { x, y },
                apply_tint(map_px_col, tint),
            );
        }
    }
}

fn set_pixel(buffer: &mut [u8], mut image_size: UVec2, pos: UVec2, value: Vec4) {
    image_size.x *= 4;
    let index = (pos.y * image_size.x + pos.x * 4) as usize;
    buffer[index] = (value[0] * 255.) as u8;
    buffer[index + 1] = (value[1] * 255.) as u8;
    buffer[index + 2] = (value[2] * 255.) as u8;
    buffer[index + 3] = (value[3] * 255.) as u8;
}

fn get_pixel(buffer: &[u8], mut image_size: UVec2, pos: UVec2) -> Vec4 {
    image_size.x *= 4;
    let index = (pos.y * image_size.x + pos.x * 4) as usize;
    Vec4::new(
        buffer[index] as f32 / 255.,
        buffer[index + 1] as f32 / 255.,
        buffer[index + 2] as f32 / 255.,
        buffer[index + 3] as f32 / 255.,
    )
}

fn alpha_blend(a: Vec4, b: Vec4, opacity: f32) -> Vec4 {
    let a = Vec4::new(a[0], a[1], a[2], a[3]);
    let b = Vec4::new(b[0], b[1], b[2], b[3]);
    a.lerp(b, opacity)
}

fn apply_tint(color: Vec4, tint_linear: Vec4) -> Vec4 {
    color * tint_linear.xyz().powf(2.2).extend(tint_linear.w)
}
