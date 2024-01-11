use bevy::{
    app::{Plugin, Update},
    asset::{load_internal_asset, AssetServer, Assets, Handle},
    ecs::{
        component::Component,
        entity::Entity,
        event::EventWriter,
        query::{Added, With},
        system::{Commands, NonSend, ParallelCommands, Query, Res, ResMut},
    },
    hierarchy::DespawnRecursiveExt,
    log::error,
    math::{UVec2, Vec2},
    reflect::Reflect,
    render::{mesh::Mesh, render_resource::Shader},
    sprite::{Material2dPlugin, Sprite, SpriteBundle, TextureAtlas},
    transform::components::Transform,
};

use crate::ldtk::{
    components::{LayerIid, WorldIid},
    json::{
        field::FieldInstance,
        level::{EntityInstance, ImagePosition, Neighbour, TileInstance},
        EntityRef, GridPoint, LdtkColor, Toc, World,
    },
    resources::{LdtkAssets, LdtkPatterns, LdtkTocs},
    sprite::{AtlasRect, NineSliceBorders, SpriteMesh},
};

use self::{
    components::{EntityIid, GlobalEntity, LdtkEntityTempTransform, LdtkLoadedLevel, LevelIid},
    entities::LdtkEntityRegistry,
    events::{LdtkEvent, LevelEvent},
    json::{
        definitions::LayerType,
        level::{LayerInstance, Level},
        LdtkJson, WorldLayout,
    },
    layer::LdtkLayers,
    resources::LdtkLevelManager,
    sprite::LdtkEntityMaterial,
};

pub mod app_ext;
pub mod components;
pub mod entities;
pub mod enums;
pub mod events;
pub mod json;
pub mod layer;
pub mod resources;
pub mod sprite;

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

        app.add_plugins(Material2dPlugin::<LdtkEntityMaterial>::default());

        app.add_systems(
            Update,
            (
                load_ldtk_json,
                unload_ldtk_level,
                global_entity_registerer,
                ldtk_temp_tranform_applier,
            ),
        );

        app.insert_non_send_resource(LdtkEntityRegistry::default());

        app.init_resource::<LdtkLevelManager>()
            .init_resource::<LdtkAssets>()
            .init_resource::<LdtkPatterns>()
            .init_resource::<LdtkTocs>();

        app.add_event::<LdtkEvent>();

        app.register_type::<LdtkLoadedLevel>()
            .register_type::<GlobalEntity>()
            .register_type::<EntityIid>()
            .register_type::<LayerIid>()
            .register_type::<LevelIid>()
            .register_type::<WorldIid>()
            .register_type::<LevelEvent>()
            .register_type::<LdtkLoader>()
            .register_type::<LdtkUnloader>()
            .register_type::<LdtkLoaderMode>()
            .register_type::<AtlasRect>()
            .register_type::<LdtkEntityMaterial>()
            .register_type::<NineSliceBorders>()
            .register_type::<SpriteMesh>();

        app.register_type::<FieldInstance>()
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
            .register_type::<GridPoint>();

        app.register_type::<LdtkLevelManager>()
            .register_type::<LdtkAssets>()
            .register_type::<LdtkPatterns>();

        #[cfg(feature = "algorithm")]
        {
            app.init_resource::<resources::LdtkWfcManager>();

            app.register_type::<resources::LdtkWfcManager>();
        }

        #[cfg(any(feature = "physics_xpbd", feature = "physics_rapier"))]
        {
            app.register_type::<layer::physics::LdtkPhysicsLayer>();
            app.register_type::<layer::physics::LdtkPhysicsAabbs>();
        }
    }
}

fn global_entity_registerer(
    mut manager: ResMut<LdtkLevelManager>,
    query: Query<(Entity, &EntityIid), Added<GlobalEntity>>,
) {
    for (entity, iid) in query.iter() {
        manager.global_entities.insert(iid.0.clone(), entity);
    }
}

fn ldtk_temp_tranform_applier(
    commands: ParallelCommands,
    mut entities_query: Query<(
        Entity,
        &mut Transform,
        &LdtkEntityTempTransform,
        Option<&GlobalEntity>,
    )>,
) {
    entities_query
        .par_iter_mut()
        .for_each(|(entity, mut transform, ldtk_temp, global_entity)| {
            if global_entity.is_some() {
                transform.translation += ldtk_temp.level_translation.extend(ldtk_temp.z_index);
            }
            commands.command_scope(|mut c| {
                c.entity(entity).remove::<LdtkEntityTempTransform>();
            });
        });
}

#[derive(Reflect, Default, Clone, Copy, PartialEq, Eq)]
pub enum LdtkLoaderMode {
    #[default]
    Tilemap,
    MapPattern,
}

#[derive(Component, Reflect, Default)]
pub struct LdtkLoader {
    pub(crate) level: String,
    pub(crate) mode: LdtkLoaderMode,
    pub(crate) trans_ovrd: Option<Vec2>,
}

#[derive(Component, Reflect, Default)]
pub struct LdtkUnloader;

pub fn unload_ldtk_level(
    mut commands: Commands,
    mut query: Query<(Entity, &LdtkLoadedLevel), With<LdtkUnloader>>,
    mut ldtk_events: EventWriter<LdtkEvent>,
) {
    query.iter_mut().for_each(|(entity, level)| {
        ldtk_events.send(LdtkEvent::LevelUnloaded(LevelEvent {
            identifier: level.identifier.clone(),
            iid: level.iid.clone(),
        }));
        commands.entity(entity).despawn_recursive();
    });
}

pub fn load_ldtk_json(
    mut commands: Commands,
    loader_query: Query<(Entity, &LdtkLoader)>,
    asset_server: Res<AssetServer>,
    ident_mapper: NonSend<LdtkEntityRegistry>,
    mut atlas_assets: ResMut<Assets<TextureAtlas>>,
    mut ldtk_events: EventWriter<LdtkEvent>,
    mut manager: ResMut<LdtkLevelManager>,
    mut assets: ResMut<LdtkAssets>,
    mut entity_material_assets: ResMut<Assets<LdtkEntityMaterial>>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut patterns: ResMut<LdtkPatterns>,
) {
    for (entity, loader) in loader_query.iter() {
        assets.initialize(
            &manager,
            &asset_server,
            &mut atlas_assets,
            &mut entity_material_assets,
            &mut mesh_assets,
        );

        let loaded = load_levels(
            &mut commands,
            &mut manager,
            loader,
            &asset_server,
            &ident_mapper,
            entity,
            &mut ldtk_events,
            &mut assets,
            &mut patterns,
        );

        if loader.mode == LdtkLoaderMode::MapPattern {
            continue;
        }

        let mut e_cmd = commands.entity(entity);
        if let Some(loaded) = loaded {
            e_cmd.insert((LevelIid(loaded.iid.clone()), loaded));
        } else {
            manager.loaded_levels.remove(&loader.level);
            error!("Failed to load level: {}!", loader.level);
        }
        e_cmd.remove::<LdtkLoader>();
    }
}

fn load_levels(
    commands: &mut Commands,
    manager: &mut LdtkLevelManager,
    loader: &LdtkLoader,
    asset_server: &AssetServer,
    ident_mapper: &LdtkEntityRegistry,
    level_entity: Entity,
    ldtk_events: &mut EventWriter<LdtkEvent>,
    assets: &mut LdtkAssets,
    patterns: &mut LdtkPatterns,
) -> Option<LdtkLoadedLevel> {
    let ldtk_data = manager.get_cached_data();

    for (level_index, level) in ldtk_data.levels.iter().enumerate() {
        // this cannot cannot be replaced by filter(), because the level_index matters.
        if level.identifier != loader.level {
            continue;
        }

        let translation = loader
            .trans_ovrd
            .unwrap_or_else(|| get_level_translation(&ldtk_data, level_index));

        let level_px = UVec2 {
            x: level.px_wid as u32,
            y: level.px_hei as u32,
        };

        let background = load_background(level, level_px, asset_server, &manager);

        #[cfg(any(feature = "physics_xpbd", feature = "physics_rapier"))]
        let mut collider_aabbs = None;
        #[cfg(feature = "algorithm")]
        let mut path_tilemap = None;

        let mut layer_layers = LdtkLayers::new(
            level.identifier.clone(),
            level_entity,
            level.layer_instances.len(),
            &assets,
            translation,
            manager.z_index,
            loader.mode,
            background,
        );

        for (layer_index, layer) in level.layer_instances.iter().enumerate() {
            #[cfg(feature = "algorithm")]
            if let Some(path) = manager.path_layer.as_ref() {
                if layer.identifier == path.identifier {
                    path_tilemap = Some(layer::path::analyze_path_layer(layer, path));
                    continue;
                }
            }

            #[cfg(any(feature = "physics_xpbd", feature = "physics_rapier"))]
            if let Some(phy) = manager.physics_layer.as_ref() {
                if layer.identifier == phy.identifier {
                    collider_aabbs = Some(layer::physics::analyze_physics_layer(layer, phy));
                    continue;
                }
            }

            load_layer(
                commands,
                level_entity,
                layer_index,
                layer,
                &mut layer_layers,
                translation,
                &ident_mapper,
                asset_server,
                &manager,
                &assets,
            );
        }

        #[cfg(any(feature = "physics_xpbd", feature = "physics_rapier"))]
        if let Some(aabbs) = collider_aabbs {
            layer_layers.assign_physics_layer(manager.physics_layer.clone().unwrap(), aabbs);
        }
        #[cfg(feature = "algorithm")]
        if let Some(path_tilemap) = path_tilemap {
            layer_layers.assign_path_layer(manager.path_layer.clone().unwrap(), path_tilemap);
        }
        layer_layers.apply_all(commands, patterns);

        ldtk_events.send(LdtkEvent::LevelLoaded(LevelEvent {
            identifier: level.identifier.clone(),
            iid: level.iid.clone(),
        }));

        return Some(LdtkLoadedLevel {
            identifier: level.identifier.clone(),
            iid: level.iid.clone(),
        });
    }

    None
}

fn load_background(
    level: &Level,
    level_px: UVec2,
    asset_server: &AssetServer,
    manager: &LdtkLevelManager,
) -> SpriteBundle {
    let texture = level
        .bg_rel_path
        .as_ref()
        .map(|path| asset_server.load(format!("{}{}", manager.asset_path_prefix, path)));

    SpriteBundle {
        sprite: Sprite {
            color: level.bg_color.into(),
            custom_size: Some(level_px.as_vec2()),
            ..Default::default()
        },
        texture: texture.unwrap_or_default(),
        transform: Transform::from_xyz(
            level_px.x as f32 / 2.,
            -(level_px.y as f32) / 2.,
            manager.z_index as f32 - level.layer_instances.len() as f32 - 1.,
        ),
        ..Default::default()
    }
}

fn load_layer(
    commands: &mut Commands,
    level_entity: Entity,
    layer_index: usize,
    layer: &LayerInstance,
    layer_grid: &mut LdtkLayers,
    translation: Vec2,
    ident_mapper: &LdtkEntityRegistry,
    asset_server: &AssetServer,
    manager: &LdtkLevelManager,
    assets: &LdtkAssets,
) {
    match layer.ty {
        LayerType::IntGrid | LayerType::AutoLayer => {
            for tile in layer.auto_layer_tiles.iter() {
                layer_grid.set(layer_index, layer, tile);
            }
        }
        LayerType::Entities => {
            for entity_instance in layer.entity_instances.iter() {
                if manager.global_entities.contains_key(&entity_instance.iid) {
                    continue;
                }

                let phantom_entity = {
                    if let Some(e) = ident_mapper.get(&entity_instance.identifier) {
                        e
                    } else if !manager.ignore_unregistered_entities {
                        panic!(
                            "Could not find entity type with entity identifier: {}! \
                            You need to register it using App::register_ldtk_entity::<T>() first!",
                            entity_instance.identifier
                        );
                    } else {
                        continue;
                    }
                };

                let mut new_entity = commands.spawn((
                    LdtkEntityTempTransform {
                        level_translation: translation,
                        z_index: manager.z_index as f32 - layer_index as f32,
                    },
                    EntityIid(entity_instance.iid.clone()),
                ));
                let mut fields = entity_instance
                    .field_instances
                    .iter()
                    .map(|field| (field.identifier.clone(), field.clone()))
                    .collect();
                phantom_entity.spawn(
                    level_entity,
                    &mut new_entity,
                    entity_instance,
                    &mut fields,
                    asset_server,
                    manager,
                    assets,
                );
            }
        }
        LayerType::Tiles => {
            for tile in layer.grid_tiles.iter() {
                layer_grid.set(layer_index, layer, tile);
            }
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
