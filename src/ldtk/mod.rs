use bevy::{
    app::{Plugin, Update},
    asset::{load_internal_asset, AssetServer, Assets, Handle},
    ecs::{
        component::Component,
        entity::Entity,
        event::EventWriter,
        query::{Added, With},
        system::{Commands, NonSend, Query, Res, ResMut},
    },
    hierarchy::{BuildChildren, DespawnRecursiveExt},
    log::error,
    math::{UVec2, Vec2},
    prelude::SpatialBundle,
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
    resources::LdtkAssets,
    sprite::{AtlasRect, NineSliceBorders, SpriteMesh},
};

use self::{
    components::{EntityIid, GlobalEntity, LdtkLoadedLevel, LevelIid},
    entities::LdtkEntityRegistry,
    events::{LdtkEvent, LevelEvent},
    json::{
        definitions::LayerType,
        level::{LayerInstance, Level},
        LdtkJson, WorldLayout,
    },
    layer::LdtkLayers,
    physics::analyze_physics_layer,
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
pub mod physics;
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
            (load_ldtk_json, unload_ldtk_level, global_entity_registerer),
        );

        app.insert_non_send_resource(LdtkEntityRegistry::default());

        app.init_resource::<LdtkLevelManager>();

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
            .register_type::<LdtkAssets>()
            .register_type::<LdtkLevelManager>()
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

#[derive(Component, Reflect, Default, Debug)]
pub struct LdtkLoader {
    pub(crate) level: String,
}

#[derive(Component, Reflect, Default, Debug)]
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
    mut entity_material_assets: ResMut<Assets<LdtkEntityMaterial>>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
) {
    manager.initialize_assets(
        &asset_server,
        &mut atlas_assets,
        &mut entity_material_assets,
        &mut mesh_assets,
    );

    for (entity, loader) in loader_query.iter() {
        let loaded = load_levels(
            &mut commands,
            &mut manager,
            loader,
            &asset_server,
            &ident_mapper,
            entity,
            &mut ldtk_events,
        );

        let mut e_cmd = commands.entity(entity);
        if let Some(loaded) = loaded {
            e_cmd.insert((
                SpatialBundle::default(),
                LevelIid(loaded.iid.clone()),
                loaded,
            ));
        } else {
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
) -> Option<LdtkLoadedLevel> {
    let ldtk_data = manager.get_cached_data();

    for (level_index, level) in ldtk_data.levels.iter().enumerate() {
        // this cannot cannot be replaced by filter(), because the level_index matters.
        if level.identifier != loader.level {
            continue;
        }

        let translation = get_level_translation(&ldtk_data, level_index, &manager);

        let level_px = UVec2 {
            x: level.px_wid as u32,
            y: level.px_hei as u32,
        };

        load_background(
            commands,
            level_entity,
            level,
            translation,
            level_px,
            asset_server,
            &manager,
        );

        let mut collider_aabbs = None;
        let mut layer_grid = LdtkLayers::new(
            level_entity,
            level.layer_instances.len(),
            level_px,
            &manager.ldtk_assets,
            translation,
            manager.z_index,
        );
        for (layer_index, layer) in level.layer_instances.iter().enumerate() {
            if let Some(phy) = manager.physics_layer.as_ref() {
                if layer.identifier == phy.identifier {
                    collider_aabbs = Some(analyze_physics_layer(layer, phy));
                    continue;
                }
            }

            load_layer(
                commands,
                level_entity,
                layer_index,
                layer,
                &mut layer_grid,
                &ident_mapper,
                asset_server,
                &manager,
            );
        }

        if let Some(aabbs) = collider_aabbs {
            layer_grid.apply_physics_layer(
                commands,
                manager.physics_layer.as_ref().unwrap(),
                aabbs,
            );
        }
        layer_grid.apply_all(commands);
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
    commands: &mut Commands,
    level_entity: Entity,
    level: &Level,
    translation: Vec2,
    level_px: UVec2,
    asset_server: &AssetServer,
    manager: &LdtkLevelManager,
) {
    let texture = match level.bg_rel_path.as_ref() {
        Some(path) => asset_server.load(format!("{}{}", manager.asset_path_prefix, path)),
        None => Handle::default(),
    };

    let bg_entity = commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: level.bg_color.into(),
                custom_size: Some(level_px.as_vec2()),
                ..Default::default()
            },
            texture,
            transform: Transform::from_translation(
                (translation + level_px.as_vec2() / 2.)
                    .extend(manager.z_index as f32 - level.layer_instances.len() as f32 - 1.),
            ),
            ..Default::default()
        })
        .id();
    commands.entity(level_entity).add_child(bg_entity);
}

fn load_layer(
    commands: &mut Commands,
    level_entity: Entity,
    layer_index: usize,
    layer: &LayerInstance,
    layer_grid: &mut LdtkLayers,
    ident_mapper: &LdtkEntityRegistry,
    asset_server: &AssetServer,
    manager: &LdtkLevelManager,
) {
    match layer.ty {
        LayerType::IntGrid | LayerType::AutoLayer => {
            for tile in layer.auto_layer_tiles.iter() {
                layer_grid.set(commands, layer_index, layer, tile);
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

                let mut new_entity = commands.spawn(EntityIid(entity_instance.iid.clone()));
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
                );
            }
        }
        LayerType::Tiles => {
            for tile in layer.grid_tiles.iter() {
                layer_grid.set(commands, layer_index, layer, tile);
            }
        }
    }
}

fn get_level_translation(ldtk_data: &LdtkJson, index: usize, manager: &LdtkLevelManager) -> Vec2 {
    let level = &ldtk_data.levels[index];
    match ldtk_data.world_layout.unwrap() {
        WorldLayout::GridVania | WorldLayout::Free => Vec2 {
            x: level.world_x as f32,
            y: (-level.world_y - level.px_hei) as f32,
        },
        WorldLayout::LinearHorizontal => {
            let mut offset = 0;
            for i in 0..index {
                offset += ldtk_data.levels[i].px_wid + manager.level_spacing.unwrap();
            }
            Vec2 {
                x: offset as f32,
                y: 0.,
            }
        }
        WorldLayout::LinearVertical => {
            let mut offset = 0;
            for i in 0..index {
                offset += ldtk_data.levels[i].px_hei + manager.level_spacing.unwrap();
            }
            Vec2 {
                x: 0.,
                y: -offset as f32,
            }
        }
    }
}
