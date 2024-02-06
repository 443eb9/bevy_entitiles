use bevy::{
    asset::{Asset, Assets, Handle},
    ecs::{
        component::Component,
        entity::Entity,
        query::With,
        system::{Commands, Query, ResMut, Resource},
    },
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderRef},
};

pub trait TilemapMaterial: Default + Asset + AsBindGroup + TypePath + Clone {
    fn vertex_shader() -> ShaderRef {
        super::TILEMAP_SHADER.into()
    }

    fn fragment_shader() -> ShaderRef {
        super::TILEMAP_SHADER.into()
    }
}

#[derive(Component, Default, Debug, Clone)]
pub struct WaitForStandardMaterialRepacement;

#[derive(Resource, Default)]
pub struct StandardTilemapMaterialSingleton(pub Option<Handle<StandardTilemapMaterial>>);

#[derive(Default, Asset, AsBindGroup, TypePath, Clone)]
pub struct StandardTilemapMaterial {}

impl TilemapMaterial for StandardTilemapMaterial {
    fn vertex_shader() -> ShaderRef {
        super::TILEMAP_SHADER.into()
    }

    fn fragment_shader() -> ShaderRef {
        super::TILEMAP_SHADER.into()
    }
}

pub fn standard_material_register(
    mut commands: Commands,
    mut tilemaps_query: Query<
        (Entity, &mut Handle<StandardTilemapMaterial>),
        With<WaitForStandardMaterialRepacement>,
    >,
    mut materials: ResMut<Assets<StandardTilemapMaterial>>,
    mut material_singleton: ResMut<StandardTilemapMaterialSingleton>,
) {
    if material_singleton.0.is_none() {
        let material = materials.add(StandardTilemapMaterial::default());
        material_singleton.0 = Some(material);
    }

    for (entity, mut material) in tilemaps_query.iter_mut() {
        *material = material_singleton.0.clone().unwrap();
        commands
            .entity(entity)
            .remove::<WaitForStandardMaterialRepacement>();
    }
}
