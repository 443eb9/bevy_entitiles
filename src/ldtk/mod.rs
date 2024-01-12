use std::path::Path;

use bevy::{
    app::{Plugin, Update},
    asset::{load_internal_asset, AssetServer, Assets, Handle},
    ecs::{
        entity::Entity,
        event::EventWriter,
        query::{Added, With},
        system::{Commands, NonSend, ParallelCommands, Query, Res, ResMut},
    },
    math::{UVec2, Vec2},
    render::{mesh::Mesh, render_resource::Shader},
    sprite::{Material2dPlugin, Sprite, SpriteBundle, TextureAtlas},
    transform::components::Transform,
};

use crate::{
    ldtk::{
        components::{LayerIid, LdtkLoader, LdtkLoaderMode, LdtkUnloader, WorldIid},
        json::{
            field::FieldInstance,
            level::{EntityInstance, ImagePosition, Neighbour, TileInstance},
            EntityRef, GridPoint, LdtkColor, Toc, World,
        },
        resources::{LdtkAssets, LdtkPatterns, LdtkTocs},
        sprite::{AtlasRect, NineSliceBorders, SpriteMesh},
    },
    tilemap::map::TilemapStorage,
};

use self::{
    components::{
        EntityIid, GlobalEntity, LdtkEntityTempTransform, LdtkLoadedLevel, LdtkUnloadLayer,
        LevelIid,
    },
    entities::{LdtkEntityRegistry, PackedLdtkEntity},
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
                unload_ldtk_layer,
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

pub fn unload_ldtk_level(
    mut commands: Commands,
    mut query: Query<(Entity, &LdtkLoadedLevel, &LevelIid), With<LdtkUnloader>>,
    mut ldtk_events: EventWriter<LdtkEvent>,
) {
    query.iter_mut().for_each(|(entity, level, iid)| {
        ldtk_events.send(LdtkEvent::LevelUnloaded(LevelEvent {
            identifier: level.identifier.clone(),
            iid: iid.0.clone(),
        }));
        level.unload(&mut commands);
        commands.entity(entity).despawn();
    });
}

pub fn unload_ldtk_layer(
    mut commands: Commands,
    mut query: Query<&mut TilemapStorage, With<LdtkUnloadLayer>>,
) {
    query.iter_mut().for_each(|mut storage| {
        storage.despawn(&mut commands);
    });
}

pub fn load_ldtk_json(
    mut commands: Commands,
    loader_query: Query<(Entity, &LdtkLoader)>,
    asset_server: Res<AssetServer>,
    entity_registry: NonSend<LdtkEntityRegistry>,
    mut atlas_assets: ResMut<Assets<TextureAtlas>>,
    mut ldtk_events: EventWriter<LdtkEvent>,
    mut manager: ResMut<LdtkLevelManager>,
    mut ldtk_assets: ResMut<LdtkAssets>,
    mut entity_material_assets: ResMut<Assets<LdtkEntityMaterial>>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut patterns: ResMut<LdtkPatterns>,
) {
    for (entity, loader) in loader_query.iter() {
        ldtk_assets.initialize(
            &manager,
            &asset_server,
            &mut atlas_assets,
            &mut entity_material_assets,
            &mut mesh_assets,
        );

        load_levels(
            &mut commands,
            &mut manager,
            loader,
            &asset_server,
            &entity_registry,
            entity,
            &mut ldtk_events,
            &mut ldtk_assets,
            &mut patterns,
        );

        commands.entity(entity).remove::<LdtkLoader>();
    }
}

fn load_levels(
    commands: &mut Commands,
    manager: &mut LdtkLevelManager,
    loader: &LdtkLoader,
    asset_server: &AssetServer,
    entity_registry: &LdtkEntityRegistry,
    level_entity: Entity,
    ldtk_events: &mut EventWriter<LdtkEvent>,
    ldtk_assets: &mut LdtkAssets,
    patterns: &mut LdtkPatterns,
) {
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

        let mut ldtk_layers = LdtkLayers::new(
            level_entity,
            level.layer_instances.len(),
            &ldtk_assets,
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

            load_layer(layer_index, layer, &mut ldtk_layers, translation, &manager);
        }

        #[cfg(any(feature = "physics_xpbd", feature = "physics_rapier"))]
        if let Some(aabbs) = collider_aabbs {
            ldtk_layers.assign_physics_layer(manager.physics_layer.clone().unwrap(), aabbs);
        }
        #[cfg(feature = "algorithm")]
        if let Some(path_tilemap) = path_tilemap {
            ldtk_layers.assign_path_layer(manager.path_layer.clone().unwrap(), path_tilemap);
        }
        ldtk_layers.apply_all(
            commands,
            patterns,
            level,
            entity_registry,
            &manager,
            &ldtk_assets,
            asset_server,
        );

        ldtk_events.send(LdtkEvent::LevelLoaded(LevelEvent {
            identifier: level.identifier.clone(),
            iid: level.iid.clone(),
        }));
    }
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
        .map(|path| asset_server.load(Path::new(&manager.asset_path_prefix).join(path)));

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
    layer_index: usize,
    layer: &LayerInstance,
    ldtk_layers: &mut LdtkLayers,
    translation: Vec2,
    manager: &LdtkLevelManager,
) {
    match layer.ty {
        LayerType::IntGrid | LayerType::AutoLayer => {
            for tile in layer.auto_layer_tiles.iter() {
                ldtk_layers.set_tile(layer_index, layer, tile);
            }
        }
        LayerType::Entities => {
            for entity_instance in layer.entity_instances.iter() {
                if manager.global_entities.contains_key(&entity_instance.iid) {
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
                    iid: EntityIid(entity_instance.iid.clone()),
                    transform: LdtkEntityTempTransform {
                        level_translation: translation,
                        z_index: manager.z_index as f32 - layer_index as f32,
                    },
                };
                ldtk_layers.set_entity(packed_entity);
            }
        }
        LayerType::Tiles => {
            for tile in layer.grid_tiles.iter() {
                ldtk_layers.set_tile(layer_index, layer, tile);
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
