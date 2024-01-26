use bevy::{
    app::{Plugin, PreStartup, Update},
    asset::{load_internal_asset, AssetServer, Assets, Handle},
    ecs::{
        entity::Entity,
        system::{Commands, Query, Res, ResMut},
    },
    math::{IVec2, Vec2},
    render::{mesh::Mesh, render_resource::Shader},
    sprite::{Material2dPlugin, MaterialMesh2dBundle, Mesh2dHandle},
};

use crate::{
    tilemap::{
        buffers::TileBuilderBuffer,
        bundles::TilemapBundle,
        map::{
            TilePivot, TileRenderSize, TilemapAxisFlip, TilemapName, TilemapSlotSize,
            TilemapStorage, TilemapTransform, TilemapType,
        },
        tile::{TileBuilder, TileLayer},
    },
    DEFAULT_CHUNK_SIZE,
};

use self::{
    components::TiledLoader,
    resources::{
        PackedTiledTilemap, PackedTiledTileset, TiledAssets, TiledLoadConfig, TiledTilemapManger,
    },
    sprite::TiledSpriteMaterial,
    xml::{
        layer::{ColorTileLayer, ColorTileLayerData, TiledLayer},
        MapOrientation,
    },
};

pub mod components;
pub mod resources;
pub mod sprite;
pub mod traits;
pub mod xml;

pub const TILED_SPRITE_SHADER: Handle<Shader> = Handle::weak_from_u128(13584136873461368486534);

pub struct EntiTilesTiledPlugin;

impl Plugin for EntiTilesTiledPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        load_internal_asset!(
            app,
            TILED_SPRITE_SHADER,
            "tiled_sprite.wgsl",
            Shader::from_wgsl
        );

        app.add_plugins(Material2dPlugin::<TiledSpriteMaterial>::default());

        app.add_systems(PreStartup, parse_tiled_xml);

        app.init_resource::<TiledLoadConfig>()
            .init_resource::<TiledAssets>()
            .init_resource::<TiledTilemapManger>();

        app.register_type::<TiledLoadConfig>()
            .register_type::<TiledAssets>()
            .register_type::<TiledTilemapManger>();

        app.add_systems(Update, load_tiled_xml);
    }
}

fn parse_tiled_xml(mut manager: ResMut<TiledTilemapManger>, config: Res<TiledLoadConfig>) {
    manager.reload_xml(&config);
}

fn load_tiled_xml(
    mut commands: Commands,
    loaders_query: Query<(Entity, &TiledLoader)>,
    mut manager: ResMut<TiledTilemapManger>,
    config: Res<TiledLoadConfig>,
    mut tiled_assets: ResMut<TiledAssets>,
    asset_server: Res<AssetServer>,
    mut material_assets: ResMut<Assets<TiledSpriteMaterial>>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
) {
    for (entity, loader) in &loaders_query {
        tiled_assets.initialize(
            &manager,
            &config,
            &asset_server,
            &mut material_assets,
            &mut mesh_assets,
        );

        load_tiled_tilemap(
            &mut commands,
            entity,
            &mut manager,
            &config,
            &tiled_assets,
            &asset_server,
            &loader,
        );

        commands.entity(entity).remove::<TiledLoader>();
    }
}

fn load_tiled_tilemap(
    commands: &mut Commands,
    tilemap_entity: Entity,
    manager: &mut TiledTilemapManger,
    config: &TiledLoadConfig,
    tiled_assets: &TiledAssets,
    asset_server: &AssetServer,
    loader: &TiledLoader,
) {
    let tiled_data = manager.get_cached_data().get(&loader.map).unwrap();

    tiled_data.xml.layers.iter().for_each(|layer| match layer {
        TiledLayer::Tiles(layer) => {
            let tile_size = Vec2::new(
                tiled_data.xml.tile_width as f32,
                tiled_data.xml.tile_height as f32,
            );
            let layer_size = IVec2::new(layer.width as i32, layer.height as i32);
            let entity = commands.spawn_empty().id();

            let mut tilemap = TilemapBundle {
                name: TilemapName(layer.name.clone()),
                tile_render_size: TileRenderSize(tile_size),
                slot_size: TilemapSlotSize(tile_size),
                ty: match tiled_data.xml.orientation {
                    MapOrientation::Orthogonal => TilemapType::Square,
                    MapOrientation::Isometric => TilemapType::Isometric,
                    MapOrientation::Staggered => TilemapType::Hexagonal(0),
                    MapOrientation::Hexagonal => {
                        TilemapType::Hexagonal(tiled_data.xml.hex_side_length)
                    }
                },
                storage: TilemapStorage::new(DEFAULT_CHUNK_SIZE, entity),
                transform: TilemapTransform::from_translation(Vec2::new(
                    layer.offset_x as f32,
                    layer.offset_y as f32,
                )),
                axis_flip: TilemapAxisFlip::Y,
                // tile_pivot: TilePivot(Vec2::new(0., 1.)),
                ..Default::default()
            };

            let mut buffer = TileBuilderBuffer::new();

            match &layer.data {
                ColorTileLayerData::Tiles(tiles) => {
                    tiles
                        .content
                        .iter_decoded(
                            layer_size,
                            tiled_assets,
                            &mut tilemap.texture,
                            &tiled_data.name,
                        )
                        .for_each(|(index, builder)| {
                            buffer.set(index, builder);
                        });
                }
                ColorTileLayerData::Chunks(chunks) => {
                    chunks.content.iter().for_each(|chunk| {
                        let offset = IVec2::new(chunk.x, -chunk.y - chunk.height as i32);
                        let size = IVec2::new(chunk.width as i32, chunk.height as i32);

                        chunk
                            .tiles
                            .iter_decoded(
                                size,
                                tiled_assets,
                                &mut tilemap.texture,
                                &tiled_data.name,
                            )
                            .for_each(|(index, builder)| {
                                buffer.set(index + offset, builder);
                            });
                    });
                }
            }

            tilemap
                .storage
                .fill_with_buffer(commands, IVec2::ZERO, buffer);
            commands.entity(entity).insert(tilemap);
        }
        TiledLayer::Objects(_) => {}
        TiledLayer::Image(layer) => {
            let (mesh, material) = (
                tiled_assets.clone_image_layer_mesh_handle(&tiled_data.name, layer.id),
                tiled_assets.clone_image_layer_material_handle(&tiled_data.name, layer.id),
            );

            commands.spawn(MaterialMesh2dBundle {
                mesh: Mesh2dHandle(mesh),
                material,
                ..Default::default()
            });
        }
        TiledLayer::Other => {}
    });
}
