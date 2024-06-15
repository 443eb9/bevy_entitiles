use bevy::{
    app::{Plugin, PreStartup, Update},
    asset::{load_internal_asset, AssetServer, Assets, Handle},
    ecs::{
        entity::Entity,
        query::With,
        system::{Commands, NonSend, Query, Res, ResMut},
    },
    log::warn,
    math::{IVec2, Vec2},
    prelude::SpatialBundle,
    render::{color::Color, mesh::Mesh, render_resource::Shader},
    sprite::{Material2dPlugin, MaterialMesh2dBundle, Mesh2dHandle},
    transform::components::Transform,
    utils::HashMap,
};

use crate::{
    render::material::StandardTilemapMaterial,
    tiled::{
        components::{TiledLoadedTilemap, TiledLoader, TiledUnloadLayer, TiledUnloader},
        resources::{PackedTiledTilemap, TiledAssets, TiledLoadConfig, TiledTilemapManger, TiledCustomTileInstance},
        sprite::TiledSpriteMaterial,
        traits::{TiledObjectRegistry, TiledCustomTileRegistry},
        xml::{
            layer::{ColorTileLayerData, TiledLayer},
            MapOrientation, TiledGroup,
        },
    },
    tilemap::{
        buffers::TileBuilderBuffer,
        bundles::StandardTilemapBundle,
        map::{
            TilePivot, TileRenderSize, TilemapAxisFlip, TilemapName, TilemapSlotSize,
            TilemapStorage, TilemapTextures, TilemapTransform, TilemapType,
        },
    },
    DEFAULT_CHUNK_SIZE,
};

pub mod app_ext;
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

        app.add_systems(
            Update,
            (unload_tiled_layer, unload_tiled_tilemap, load_tiled_xml),
        );

        app.init_non_send_resource::<TiledObjectRegistry>();
        app.init_non_send_resource::<TiledCustomTileRegistry>();
    }
}

fn parse_tiled_xml(mut manager: ResMut<TiledTilemapManger>, config: Res<TiledLoadConfig>) {
    manager.reload_xml(&config);
}

fn unload_tiled_tilemap(
    mut commands: Commands,
    tilemaps_query: Query<(Entity, &TiledLoadedTilemap), With<TiledUnloader>>,
) {
    tilemaps_query.iter().for_each(|(entity, tilemap)| {
        tilemap.unload(&mut commands);
        commands.entity(entity).despawn();
    });
}

fn unload_tiled_layer(
    mut commands: Commands,
    mut layers_query: Query<(Entity, Option<&mut TilemapStorage>), With<TiledUnloadLayer>>,
) {
    layers_query.iter_mut().for_each(|(entity, storage)| {
        if let Some(mut st) = storage {
            st.despawn(&mut commands);
        } else {
            commands.entity(entity).despawn();
        }
    });
}

fn load_tiled_xml(
    mut commands: Commands,
    loaders_query: Query<(Entity, &TiledLoader)>,
    mut manager: ResMut<TiledTilemapManger>,
    config: Res<TiledLoadConfig>,
    mut tiled_assets: ResMut<TiledAssets>,
    asset_server: Res<AssetServer>,
    mut sprite_material_assets: ResMut<Assets<TiledSpriteMaterial>>,
    mut tilemap_material_assets: ResMut<Assets<StandardTilemapMaterial>>,
    mut textures_assets: ResMut<Assets<TilemapTextures>>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    object_registry: NonSend<TiledObjectRegistry>,
    custom_tiles_registry: NonSend<TiledCustomTileRegistry>,
) {
    for (entity, loader) in &loaders_query {
        tiled_assets.initialize(
            &manager,
            &config,
            &asset_server,
            &mut sprite_material_assets,
            &mut textures_assets,
            &mut mesh_assets,
        );

        load_tiled_tilemap(
            &mut commands,
            &mut manager,
            &config,
            &tiled_assets,
            &asset_server,
            &loader,
            &object_registry,
            &custom_tiles_registry,
            entity,
            &mut tilemap_material_assets,
        );

        commands.entity(entity).remove::<TiledLoader>();
    }
}

fn load_tiled_tilemap(
    commands: &mut Commands,
    manager: &mut TiledTilemapManger,
    config: &TiledLoadConfig,
    tiled_assets: &TiledAssets,
    asset_server: &AssetServer,
    loader: &TiledLoader,
    object_registry: &TiledObjectRegistry,
    custom_tiles_registry: &TiledCustomTileRegistry,
    map_entity: Entity,
    tilemap_material_assets: &mut Assets<StandardTilemapMaterial>,
) {
    let tiled_data = manager.get_cached_data().get(&loader.map).unwrap();
    let mut loaded_map = TiledLoadedTilemap {
        name: tiled_data.name.clone(),
        layers: HashMap::default(),
        objects: HashMap::default(),
    };
    let mut z = config.z_index;

    tiled_data.xml.layers.iter().for_each(|layer| {
        load_layer(
            commands,
            tiled_data,
            &mut z,
            layer,
            tiled_assets,
            asset_server,
            object_registry,
            custom_tiles_registry,
            config,
            &mut loaded_map,
            tilemap_material_assets,
        )
    });

    tiled_data.xml.groups.iter().for_each(|group| {
        load_group(
            commands,
            tiled_data,
            &mut z,
            group,
            tiled_assets,
            asset_server,
            object_registry,
            custom_tiles_registry,
            config,
            &mut loaded_map,
            tilemap_material_assets,
        )
    });

    commands.entity(map_entity).insert(loaded_map);
}

fn load_group(
    commands: &mut Commands,
    tiled_data: &PackedTiledTilemap,
    z: &mut f32,
    group: &TiledGroup,
    tiled_assets: &TiledAssets,
    asset_server: &AssetServer,
    object_registry: &TiledObjectRegistry,
    custom_tiles_registry: &TiledCustomTileRegistry,
    config: &TiledLoadConfig,
    loaded_map: &mut TiledLoadedTilemap,
    tilemap_material_assets: &mut Assets<StandardTilemapMaterial>,
) {
    group.layers.iter().for_each(|content| {
        load_layer(
            commands,
            tiled_data,
            z,
            content,
            tiled_assets,
            asset_server,
            object_registry,
            custom_tiles_registry,
            config,
            loaded_map,
            tilemap_material_assets,
        )
    });

    group.groups.iter().for_each(|group| {
        load_group(
            commands,
            tiled_data,
            z,
            group,
            tiled_assets,
            asset_server,
            object_registry,
            custom_tiles_registry,
            config,
            loaded_map,
            tilemap_material_assets,
        )
    });
}

fn load_layer(
    commands: &mut Commands,
    tiled_data: &PackedTiledTilemap,
    z: &mut f32,
    layer: &TiledLayer,
    tiled_assets: &TiledAssets,
    asset_server: &AssetServer,
    object_registry: &TiledObjectRegistry,
    custom_tiles_registry: &TiledCustomTileRegistry,
    config: &TiledLoadConfig,
    loaded_map: &mut TiledLoadedTilemap,
    tilemap_material_assets: &mut Assets<StandardTilemapMaterial>,
) {
    *z += 0.1;

    match layer {
        TiledLayer::Tiles(layer) => {
            let tile_size = Vec2::new(
                tiled_data.xml.tile_width as f32,
                tiled_data.xml.tile_height as f32,
            );
            let layer_size = IVec2::new(layer.width as i32, layer.height as i32);
            let entity = commands.spawn_empty().id();
            let (textures, animations) = tiled_assets.get_tilemap_data(&loaded_map.name);
            let mut custom_properties_tiles: Vec<(IVec2,TiledCustomTileInstance)> = vec!();

            let tint = Color::rgba(
                layer.tint.r,
                layer.tint.g,
                layer.tint.b,
                layer.tint.a * layer.opacity,
            );
            let mut tilemap = StandardTilemapBundle {
                name: TilemapName(layer.name.clone()),
                slot_size: TilemapSlotSize(tile_size),
                ty: match tiled_data.xml.orientation {
                    MapOrientation::Orthogonal => TilemapType::Square,
                    MapOrientation::Isometric => TilemapType::Isometric,
                    MapOrientation::Staggered => TilemapType::Hexagonal(0),
                    MapOrientation::Hexagonal => {
                        TilemapType::Hexagonal(tiled_data.xml.hex_side_length)
                    }
                },
                transform: TilemapTransform::from_translation_3d(
                    Vec2::new(layer.offset_x as f32, layer.offset_y as f32)
                        + match tiled_data.xml.orientation {
                            MapOrientation::Orthogonal | MapOrientation::Isometric => Vec2::ZERO,
                            MapOrientation::Staggered | MapOrientation::Hexagonal => {
                                tiled_data.xml.stagger_index.get_offset() * tile_size
                            }
                        },
                    *z,
                ),
                textures,
                animations,
                material: tilemap_material_assets.add(StandardTilemapMaterial { tint }),
                axis_flip: match tiled_data.xml.orientation {
                    MapOrientation::Isometric => TilemapAxisFlip::all(),
                    _ => TilemapAxisFlip::Y,
                },
                tile_pivot: match tiled_data.xml.orientation {
                    MapOrientation::Isometric => TilePivot(Vec2::new(0.5, 0.)),
                    _ => TilePivot::default(),
                },
                ..Default::default()
            };

            let mut buffer = TileBuilderBuffer::new();
            let mut tile_render_size: Option<TileRenderSize> = None;

            match &layer.data {
                ColorTileLayerData::Tiles(tiles) => {
                    tiles
                        .content
                        .iter_decoded(layer_size, &loaded_map.name, tiled_assets, &tiled_data)
                        .for_each(|(index, builder, trs, custom_tile)| {
                            buffer.set(index, builder);
                            if let Some(t) = tile_render_size {
                                assert_eq!(
                                    t.0,
                                    trs.0,
                                    "Using tilesets that have different tile size, which is not supported currently."
                                );
                            }
                            tile_render_size = Some(trs);
                            if let Some(c) = custom_tile {
                                custom_properties_tiles.push((index, c.clone()));
                            }
                        });
                }
                ColorTileLayerData::Chunks(chunks) => {
                    chunks.content.iter().for_each(|chunk| {
                        let offset = IVec2::new(chunk.x, chunk.y);
                        let size = IVec2::new(chunk.width as i32, chunk.height as i32);

                        chunk
                            .tiles
                            .iter_decoded(size, &loaded_map.name, tiled_assets, &tiled_data)
                            .for_each(|(index, builder, trs, _)| {
                                buffer.set(index + offset, builder);
                                if let Some(t) = tile_render_size {
                                    assert_eq!(
                                        t.0,
                                        trs.0,
                                        "Using tilesets that have different tile size, which is not supported currently."
                                    );
                                }
                                tile_render_size = Some(trs);
                            });
                    });
                }
            }

            tilemap.tile_render_size = tile_render_size.unwrap_or_default();
            tilemap.storage = TilemapStorage::new(
                if tilemap.tile_render_size.0.y > tilemap.slot_size.0.y {
                    warn!(
                        "Using chunk size 1 for layer {} as it looks like a 3d tilemap.",
                        layer.name
                    );
                    1
                } else {
                    DEFAULT_CHUNK_SIZE
                },
                entity,
            );
            tilemap
                .storage
                .fill_with_buffer(commands, IVec2::ZERO, buffer);
            

            // Tiles entity have been spawned: add custom properties to the ones we registered
            custom_properties_tiles.iter().for_each(|(index, custom_tile_instance)| {
                if let Some(entity) = tilemap.storage.get(*index) {
                    let Some(phantom) = custom_tiles_registry.get(&custom_tile_instance.ty) else {
                        if config.ignore_unregisterd_custom_tiles {
                            return;
                        }
                        panic!(
                            "Could not find component type with custom class identifier: {}! \
                        You need to register it using App::register_tiled_custom_tile::<T>() first!",
                            custom_tile_instance.ty
                        )
                    };
                    
                    let mut entity = commands.entity(entity);
                    phantom.initialize(
                        &mut entity,
                        custom_tile_instance,
                        &custom_tile_instance
                            .properties
                            .instances
                            .iter()
                            .map(|inst| (inst.ty.clone(), inst.clone()))
                            .collect(),
                        asset_server,
                        tiled_assets,
                        tiled_data.name.clone(),
                    );
                }
            });
            commands.entity(entity).insert(tilemap);
            loaded_map.layers.insert(layer.id, entity);
        }
        TiledLayer::Objects(layer) => {
            let num_objects = layer.objects.len();
            layer
                .objects
                .iter()
                .enumerate()
                .for_each(|(index, object)| {
                    let Some(phantom) = object_registry.get(&object.ty) else {
                        if config.ignore_unregisterd_objects {
                            return;
                        }
                        panic!(
                            "Could not find component type with custom class identifier: {}! \
                        You need to register it using App::register_tiled_object::<T>() first!",
                            object.ty
                        )
                    };

                    let mut entity = commands.spawn_empty();
                    phantom.initialize(
                        &mut entity,
                        object,
                        &object
                            .properties
                            .instances
                            .iter()
                            .map(|inst| (inst.ty.clone(), inst.clone()))
                            .collect(),
                        asset_server,
                        tiled_assets,
                        tiled_data.name.clone(),
                    );
                    entity.insert(SpatialBundle {
                        transform: Transform::from_xyz(
                            object.x + object.width / 2.,
                            -object.y - object.height / 2.,
                            *z + index as f32 / (num_objects + 1) as f32,
                        ),
                        ..Default::default()
                    });

                    loaded_map.objects.insert(object.id, entity.id());
                });
        }
        TiledLayer::Image(layer) => {
            let ((mesh, z), material) = (
                tiled_assets.clone_image_layer_mesh_handle(&tiled_data.name, layer.id),
                tiled_assets.clone_image_layer_material_handle(&tiled_data.name, layer.id),
            );

            let entity = commands
                .spawn(MaterialMesh2dBundle {
                    mesh: Mesh2dHandle(mesh),
                    material,
                    transform: Transform::from_xyz(0., 0., z),
                    ..Default::default()
                })
                .id();

            loaded_map.layers.insert(layer.id, entity);
        }
        TiledLayer::Other => {}
    }
}
