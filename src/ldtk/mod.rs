use std::path::Path;

use bevy::{
    app::{Plugin, Update},
    asset::{load_internal_asset, AssetApp, AssetEvent, AssetId, AssetServer, Assets, Handle},
    ecs::{
        entity::Entity,
        query::{Added, With},
        system::{Commands, NonSend, ParallelCommands, Query, Res, ResMut},
    },
    log::{error, info, warn},
    math::{UVec2, Vec2},
    prelude::{EventReader, Local},
    render::{mesh::Mesh, render_resource::Shader},
    sprite::{Material2dPlugin, Sprite, SpriteBundle, TextureAtlasLayout},
    transform::components::Transform,
    utils::Entry,
};

use crate::{
    ldtk::{
        components::{
            EntityIid, GlobalEntity, LayerIid, LdtkLoadedLevel, LdtkTempTransform, LdtkUnloadLayer,
            LevelIid, WorldIid,
        },
        events::{LdtkLevel, LdtkLevelEvent, LdtkLevelLoader},
        json::{
            definitions::LayerType,
            field::FieldInstance,
            level::{EntityInstance, ImagePosition, LayerInstance, Level, Neighbour, TileInstance},
            EntityRef, GridPoint, LdtkColor, LdtkJson, Toc, World, WorldLayout,
        },
        layer::{LdtkLayers, PackedLdtkEntity},
        resources::{
            LdtkAdditionalLayers, LdtkAssets, LdtkGlobalEntityRegistry, LdtkJsonLoader,
            LdtkJsonToAssets, LdtkLevelConfig, LdtkLevelIdentifierToIid, LdtkLoadedLevels,
            LdtkPatterns, LdtkTocs,
        },
        sprite::{AtlasRect, LdtkEntityMaterial, NineSliceBorders, SpriteMesh},
        traits::{LdtkEntityRegistry, LdtkEntityTagRegistry},
    },
    render::material::StandardTilemapMaterial,
    tilemap::map::{TilemapStorage, TilemapTextures},
};

#[cfg(feature = "algorithm")]
use crate::algorithm::pathfinding::PathTilemaps;

pub mod app_ext;
pub mod components;
pub mod events;
pub mod json;
pub mod layer;
pub mod resources;
pub mod sprite;
pub mod traits;

pub const ENTITY_SPRITE_SHADER: Handle<Shader> = Handle::weak_from_u128(89874656485416351634163551);

pub struct EntiTilesLdtkPlugin;

impl Plugin for EntiTilesLdtkPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        load_internal_asset!(
            app,
            ENTITY_SPRITE_SHADER,
            "entity_sprite.wgsl",
            Shader::from_wgsl
        );

        app.add_plugins(Material2dPlugin::<LdtkEntityMaterial>::default())
            .add_systems(
                Update,
                (
                    ldtk_asset_events_handler,
                    load_ldtk_level,
                    unload_ldtk_level,
                    unload_ldtk_layer,
                    global_entity_registerer,
                    ldtk_temp_tranform_applier,
                    apply_ldtk_layers,
                ),
            )
            .insert_non_send_resource(LdtkEntityRegistry::default())
            .init_asset::<LdtkJson>()
            .init_asset::<LdtkAssets>()
            .init_asset_loader::<LdtkJsonLoader>()
            .init_resource::<LdtkLoadedLevels>()
            .init_resource::<LdtkLevelConfig>()
            .init_resource::<LdtkAdditionalLayers>()
            .init_resource::<LdtkJsonToAssets>()
            .init_resource::<LdtkPatterns>()
            .init_resource::<LdtkTocs>()
            .init_resource::<LdtkGlobalEntityRegistry>()
            .init_resource::<LdtkLevelIdentifierToIid>()
            .add_event::<LdtkLevelEvent>()
            .register_type::<LdtkLoadedLevel>()
            .register_type::<GlobalEntity>()
            .register_type::<EntityIid>()
            .register_type::<LayerIid>()
            .register_type::<LevelIid>()
            .register_type::<WorldIid>()
            .register_type::<AtlasRect>()
            .register_type::<LdtkEntityMaterial>()
            .register_type::<NineSliceBorders>()
            .register_type::<SpriteMesh>()
            .register_type::<FieldInstance>()
            .register_type::<Level>()
            .register_type::<ImagePosition>()
            .register_type::<Neighbour>()
            .register_type::<LayerInstance>()
            .register_type::<TileInstance>()
            .register_type::<EntityInstance>()
            .register_type::<LdtkColor>()
            .register_type::<LdtkJson>()
            .register_type::<Toc>()
            .register_type::<World>()
            .register_type::<EntityRef>()
            .register_type::<GridPoint>()
            .register_type::<LdtkLevelConfig>()
            .register_type::<LdtkAdditionalLayers>()
            .register_type::<LdtkAssets>()
            .register_type::<LdtkPatterns>()
            .register_type::<LdtkGlobalEntityRegistry>();

        #[cfg(feature = "algorithm")]
        {
            app.init_resource::<resources::LdtkWfcManager>()
                .register_type::<resources::LdtkWfcManager>();
        }

        #[cfg(feature = "physics")]
        {
            app.register_type::<layer::physics::LdtkPhysicsLayer>();
        }
    }
}

fn global_entity_registerer(
    mut registry: ResMut<LdtkGlobalEntityRegistry>,
    query: Query<(Entity, &EntityIid), Added<GlobalEntity>>,
) {
    query.iter().for_each(|(entity, iid)| {
        registry.insert(iid.clone(), entity);
    });
}

fn ldtk_temp_tranform_applier(
    commands: ParallelCommands,
    mut entities_query: Query<(Entity, &mut Transform, &LdtkTempTransform)>,
) {
    entities_query
        .par_iter_mut()
        .for_each(|(entity, mut transform, ldtk_temp)| {
            transform.translation += ldtk_temp.level_translation.extend(ldtk_temp.z_index);
            commands.command_scope(|mut c| {
                c.entity(entity).remove::<LdtkTempTransform>();
            });
        });
}

pub fn ldtk_asset_events_handler(
    mut asset_event: EventReader<AssetEvent<LdtkJson>>,
    mut assets: ResMut<Assets<LdtkAssets>>,
    jsons: Res<Assets<LdtkJson>>,
    mut json_to_assets: ResMut<LdtkJsonToAssets>,
    config: Res<LdtkLevelConfig>,
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut material_assets: ResMut<Assets<LdtkEntityMaterial>>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut identifier_to_id: ResMut<LdtkLevelIdentifierToIid>,
) {
    for ev in asset_event.read() {
        match ev {
            AssetEvent::Added { id } | AssetEvent::Modified { id } => {
                let json = jsons.get(*id).unwrap();
                let asset = assets.add(LdtkAssets::new(
                    &config,
                    json,
                    &asset_server,
                    &mut atlas_layouts,
                    &mut material_assets,
                    &mut mesh_assets,
                ));
                json_to_assets.0.insert(*id, asset);
                identifier_to_id.0.insert(
                    *id,
                    json.levels
                        .iter()
                        .map(|level| (level.identifier.clone(), LevelIid(level.iid.clone())))
                        .collect(),
                );
            }
            AssetEvent::Removed { id } => {
                if let Some(asset) = json_to_assets.get(id) {
                    assets.remove(asset);
                }
            }
            AssetEvent::Unused { .. } | AssetEvent::LoadedWithDependencies { .. } => {}
        }
    }
}

pub fn unload_ldtk_level(
    mut commands: Commands,
    query: Query<(Entity, &LdtkLoadedLevel)>,
    global_entities: Res<LdtkGlobalEntityRegistry>,
    mut level_events: EventReader<LdtkLevelEvent>,
    loaded_levels: Res<LdtkLoadedLevels>,
    identifier_to_iid: Res<LdtkLevelIdentifierToIid>,
) {
    for ev in level_events.read() {
        let LdtkLevelEvent::Unload(unloader) = ev else {
            continue;
        };

        let entity = loaded_levels
            .get(&unloader.json)
            .and_then(|entities| match &unloader.level {
                LdtkLevel::Identifier(ident) => identifier_to_iid
                    .get(&unloader.json)
                    .and_then(|mapper| mapper.get(ident).and_then(|iid| entities.get(iid))),
                LdtkLevel::Iid(iid) => entities.get(iid),
            });

        let Some((entity, level)) = entity.and_then(|e| query.get(*e).ok()) else {
            error!(
                "Failed to unload level: Failed to find the corresponding entity. {}",
                unloader.level
            );
            continue;
        };

        level.unload(&mut commands, &global_entities);
        commands.entity(entity).despawn();
    }
}

#[cfg(not(feature = "physics"))]
pub fn unload_ldtk_layer(
    mut commands: Commands,
    mut query: Query<&mut TilemapStorage, With<LdtkUnloadLayer>>,
) {
    query.iter_mut().for_each(|mut storage| {
        storage.despawn(&mut commands);
    });
}

#[cfg(feature = "physics")]
pub fn unload_ldtk_layer(
    mut commands: Commands,
    mut query: Query<
        (
            &mut TilemapStorage,
            Option<&mut crate::tilemap::physics::PhysicsTilemap>,
        ),
        With<LdtkUnloadLayer>,
    >,
) {
    query.iter_mut().for_each(|(mut storage, physics)| {
        if let Some(mut physics) = physics {
            physics.remove_all(&mut commands);
        }
        storage.despawn(&mut commands);
    });
}

pub fn load_ldtk_level(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    config: Res<LdtkLevelConfig>,
    addi_layers: Res<LdtkAdditionalLayers>,
    mut patterns: ResMut<LdtkPatterns>,
    global_entities: Res<LdtkGlobalEntityRegistry>,
    mut level_events: EventReader<LdtkLevelEvent>,
    ldtk_jsons: Res<Assets<LdtkJson>>,
    ldtk_assets: Res<Assets<LdtkAssets>>,
    json_to_assets: Res<LdtkJsonToAssets>,
    mut loaded_levels: ResMut<LdtkLoadedLevels>,
    mut retry_queue: Local<Vec<LdtkLevelLoader>>,
) {
    for ev in level_events.read() {
        let LdtkLevelEvent::Load(loader) = ev else {
            continue;
        };

        let Some(ldtk_data) = ldtk_jsons.get(loader.json) else {
            warn!(
                "Failed to load level: Json haven't parsed yet. Retrying next frame. {}",
                loader.level
            );
            retry_queue.push(loader.clone());
            continue;
        };

        let assets_handle = json_to_assets.get(&loader.json);
        let Some(ldtk_assets) = assets_handle.and_then(|h| ldtk_assets.get(h)) else {
            warn!(
                "Failed to load level: Assets haven't read yet. Retrying next frame. {}",
                loader.level
            );
            retry_queue.push(loader.clone());
            continue;
        };

        let level_entity = commands.spawn_empty().id();
        load_levels(
            &mut commands,
            &config,
            &ldtk_data,
            &addi_layers,
            loader,
            &asset_server,
            level_entity,
            assets_handle.unwrap().id(),
            &ldtk_assets,
            &mut patterns,
            &global_entities,
            &mut loaded_levels,
        );
        info!("Successfully loaded level. {}", loader.level);
    }
}

fn load_levels(
    commands: &mut Commands,
    config: &LdtkLevelConfig,
    ldtk_data: &LdtkJson,
    addi_layers: &LdtkAdditionalLayers,
    loader: &LdtkLevelLoader,
    asset_server: &AssetServer,
    level_entity: Entity,
    assets_id: AssetId<LdtkAssets>,
    ldtk_assets: &LdtkAssets,
    patterns: &mut LdtkPatterns,
    global_entities: &LdtkGlobalEntityRegistry,
    loaded_levels: &mut LdtkLoadedLevels,
) {
    let Some((level_index, level)) = (match &loader.level {
        LdtkLevel::Identifier(ident) => ldtk_data
            .levels
            .iter()
            .enumerate()
            .find(|(_, level)| level.identifier == *ident),
        LdtkLevel::Iid(iid) => ldtk_data
            .levels
            .iter()
            .enumerate()
            .find(|(_, level)| level.iid == **iid),
    }) else {
        error!(
            "Failed to load level: Level doesn't exist. {}",
            loader.level
        );
        return;
    };

    let loaded_levels = loaded_levels.0.entry(loader.json).or_default();
    match loaded_levels.entry(LevelIid(level.iid.clone())) {
        Entry::Occupied(_) => {
            error!(
                "Failed to load level: Level already loaded. {}",
                loader.level
            );
            return;
        }
        Entry::Vacant(e) => {
            dbg!(&loader.level);
            e.insert(level_entity);
        }
    }

    let translation = loader
        .trans_ovrd
        .unwrap_or_else(|| get_level_translation(&ldtk_data, level_index));

    let level_px = UVec2 {
        x: level.px_wid as u32,
        y: level.px_hei as u32,
    };

    let background = load_background(level, translation, level_px, asset_server, config);

    let mut ldtk_layers = LdtkLayers::new(
        level_entity,
        level,
        level.layer_instances.len(),
        assets_id,
        &ldtk_assets,
        translation,
        config.z_index,
        loader.mode,
        background,
    );

    for (layer_index, layer) in level.layer_instances.iter().enumerate() {
        #[cfg(feature = "algorithm")]
        if let Some(path) = addi_layers.path_layer.as_ref() {
            if layer.identifier == path.identifier {
                ldtk_layers
                    .assign_path_layer(path.clone(), layer::path::analyze_path_layer(layer, path));
                continue;
            }
        }

        #[cfg(feature = "physics")]
        if let Some(phy) = addi_layers.physics_layer.as_ref() {
            if layer.identifier == phy.identifier {
                ldtk_layers.assign_physics_layer(
                    phy.clone(),
                    layer.int_grid_csv.clone(),
                    UVec2 {
                        x: layer.c_wid as u32,
                        y: layer.c_hei as u32,
                    },
                );
                continue;
            }
        }

        load_layer(
            layer_index,
            layer,
            &mut ldtk_layers,
            translation,
            config,
            &global_entities,
            patterns,
            loader,
        );
    }

    commands.entity(level_entity).insert(ldtk_layers);
}

fn load_background(
    level: &Level,
    translation: Vec2,
    level_px: UVec2,
    asset_server: &AssetServer,
    config: &LdtkLevelConfig,
) -> SpriteBundle {
    let texture = level
        .bg_rel_path
        .as_ref()
        .map(|path| asset_server.load(Path::new(&config.asset_path_prefix).join(path)));

    SpriteBundle {
        sprite: Sprite {
            color: level.bg_color.into(),
            custom_size: Some(level_px.as_vec2()),
            ..Default::default()
        },
        texture: texture.unwrap_or_default(),
        transform: Transform::from_xyz(
            level_px.x as f32 / 2. + translation.x,
            -(level_px.y as f32) / 2. + translation.y,
            config.z_index as f32 - level.layer_instances.len() as f32 - 1.,
        ),
        ..Default::default()
    }
}

fn load_layer(
    layer_index: usize,
    layer: &LayerInstance,
    ldtk_layers: &mut LdtkLayers,
    translation: Vec2,
    config: &LdtkLevelConfig,
    global_entities: &LdtkGlobalEntityRegistry,
    patterns: &LdtkPatterns,
    loader: &LdtkLevelLoader,
) {
    match layer.ty {
        LayerType::IntGrid | LayerType::AutoLayer => {
            layer.auto_layer_tiles.iter().for_each(|tile| {
                ldtk_layers.set_tile(layer_index, layer, tile, config, patterns, &loader.mode);
            });
        }
        LayerType::Entities => {
            for (order, entity_instance) in layer.entity_instances.iter().enumerate() {
                let iid = EntityIid(entity_instance.iid.clone());
                if global_entities.contains_key(&iid) {
                    continue;
                }

                let fields = entity_instance
                    .field_instances
                    .iter()
                    .map(|field| (field.identifier.clone(), field.clone()))
                    .collect();
                let packed_entity = PackedLdtkEntity {
                    instance: entity_instance.clone(),
                    fields,
                    iid,
                    transform: LdtkTempTransform {
                        level_translation: translation,
                        z_index: config.z_index as f32
                            - layer_index as f32
                            - (1. - (order as f32 / layer.entity_instances.len() as f32)),
                    },
                };
                ldtk_layers.set_entity(packed_entity);
            }
        }
        LayerType::Tiles => {
            layer.grid_tiles.iter().for_each(|tile| {
                ldtk_layers.set_tile(layer_index, layer, tile, config, patterns, &loader.mode);
            });
        }
    }
}

fn get_level_translation(ldtk_data: &LdtkJson, index: usize) -> Vec2 {
    let level = &ldtk_data.levels[index];
    match ldtk_data.world_layout.unwrap() {
        WorldLayout::GridVania | WorldLayout::Free => Vec2 {
            x: level.world_x as f32,
            y: -level.world_y as f32,
        },
        WorldLayout::LinearHorizontal | WorldLayout::LinearVertical => Vec2::ZERO,
    }
}

fn apply_ldtk_layers(
    mut commands: Commands,
    mut ldtk_layers_query: Query<(Entity, &mut LdtkLayers)>,
    mut ldtk_patterns: ResMut<LdtkPatterns>,
    entity_registry: Option<NonSend<LdtkEntityRegistry>>,
    entity_tag_registry: Option<NonSend<LdtkEntityTagRegistry>>,
    config: Res<LdtkLevelConfig>,
    ldtk_assets: Res<Assets<LdtkAssets>>,
    asset_server: Res<AssetServer>,
    mut material_assets: ResMut<Assets<StandardTilemapMaterial>>,
    mut textures_assets: ResMut<Assets<TilemapTextures>>,
    #[cfg(feature = "algorithm")] mut path_tilemaps: ResMut<PathTilemaps>,
) {
    for (entity, mut ldtk_layers) in &mut ldtk_layers_query {
        let entity_registry = entity_registry.as_ref().map(|r| &**r);
        let entity_tag_registry = entity_tag_registry.as_ref().map(|r| &**r);
        let ldtk_assets = ldtk_assets.get(ldtk_layers.assets_id).unwrap();

        ldtk_layers.apply_all(
            &mut commands,
            &mut ldtk_patterns,
            &entity_registry.unwrap_or(&LdtkEntityRegistry::default()),
            &entity_tag_registry.unwrap_or(&LdtkEntityTagRegistry::default()),
            &config,
            ldtk_assets,
            &asset_server,
            &mut material_assets,
            &mut textures_assets,
            #[cfg(feature = "algorithm")]
            &mut path_tilemaps,
        );

        commands.entity(entity).remove::<LdtkLayers>();
    }
}
