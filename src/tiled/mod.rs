use bevy::{
    app::{Plugin, PreStartup, Update},
    asset::{load_internal_asset, AssetServer, Assets, Handle},
    ecs::{
        entity::Entity,
        query::With,
        system::{Commands, NonSend, Query, Res, ResMut},
    },
    math::{IVec2, Vec2, Vec4},
    prelude::SpatialBundle,
    render::{mesh::Mesh, render_resource::Shader},
    sprite::{Material2dPlugin, MaterialMesh2dBundle, Mesh2dHandle},
    transform::components::Transform,
    utils::HashMap,
};

use crate::{
    render::material::StandardTilemapMaterial,
    tiled::traits::TiledObjectRegistry,
    tilemap::{
        buffers::TileBuilderBuffer,
        bundles::StandardTilemapBundle,
        map::{
            TilePivot, TileRenderSize, TilemapAxisFlip, TilemapName, TilemapSlotSize,
            TilemapStorage, TilemapTransform, TilemapType,
        },
    },
    DEFAULT_CHUNK_SIZE,
};

use self::{
    components::{TiledLoadedTilemap, TiledLoader, TiledUnloadLayer, TiledUnloader},
    resources::{PackedTiledTilemap, TiledAssets, TiledLoadConfig, TiledTilemapManger},
    sprite::TiledSpriteMaterial,
    xml::{
        layer::{ColorTileLayerData, TiledLayer},
        MapOrientation, TiledGroup,
    },
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
    mut material_assets: ResMut<Assets<TiledSpriteMaterial>>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    object_registry: NonSend<TiledObjectRegistry>,
    mut materials: ResMut<Assets<StandardTilemapMaterial>>,
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
            &mut manager,
            &config,
            &tiled_assets,
            &asset_server,
            &loader,
            &object_registry,
            entity,
            &mut materials,
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
    map_entity: Entity,
    materials: &mut Assets<StandardTilemapMaterial>,
) {
    let tiled_data = manager.get_cached_data().get(&loader.map).unwrap();
    let mut loaded_map = TiledLoadedTilemap {
        map: tiled_data.name.clone(),
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
            config,
            &mut loaded_map,
            materials,
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
            config,
            &mut loaded_map,
            materials,
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
    config: &TiledLoadConfig,
    loaded_map: &mut TiledLoadedTilemap,
    materials: &mut Assets<StandardTilemapMaterial>,
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
            config,
            loaded_map,
            materials,
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
            config,
            loaded_map,
            materials,
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
    config: &TiledLoadConfig,
    loaded_map: &mut TiledLoadedTilemap,
    materials: &mut Assets<StandardTilemapMaterial>,
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

            let mut tilemap = StandardTilemapBundle {
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

            let tint = Vec4::new(
                layer.tint.r,
                layer.tint.g,
                layer.tint.b,
                layer.tint.a * layer.opacity,
            );
            match &layer.data {
                ColorTileLayerData::Tiles(tiles) => {
                    tiles
                        .content
                        .iter_decoded(
                            layer_size,
                            tiled_assets,
                            &mut tilemap,
                            &tiled_data,
                            tint,
                            materials,
                        )
                        .for_each(|(index, builder)| {
                            buffer.set(index, builder);
                        });
                }
                ColorTileLayerData::Chunks(chunks) => {
                    chunks.content.iter().for_each(|chunk| {
                        let offset = IVec2::new(chunk.x, chunk.y);
                        let size = IVec2::new(chunk.width as i32, chunk.height as i32);

                        chunk
                            .tiles
                            .iter_decoded(
                                size,
                                tiled_assets,
                                &mut tilemap,
                                &tiled_data,
                                tint,
                                materials,
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

            loaded_map.layers.insert(layer.id, entity);
        }
        TiledLayer::Objects(layer) => {
            layer.objects.iter().for_each(|obj| {
                let Some(phantom) = object_registry.get(&obj.ty) else {
                    if config.ignore_unregisterd_objects {
                        return;
                    }
                    panic!(
                        "Could not find component type with custom class identifier: {}! \
                        You need to register it using App::register_tiled_object::<T>() first!",
                        obj.ty
                    )
                };

                let mut entity = commands.spawn_empty();
                phantom.initialize(
                    &mut entity,
                    obj,
                    &obj.properties
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
                        obj.x + obj.width / 2.,
                        -obj.y - obj.height / 2.,
                        *z,
                    ),
                    // .with_rotation(Quat::from_rotation_z(-obj.rotation / 180. * PI)),
                    ..Default::default()
                });

                loaded_map.objects.insert(obj.id, entity.id());
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
