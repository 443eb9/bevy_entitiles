use bevy::{
    asset::Handle,
    ecs::{entity::Entity, system::Resource},
    render::{render_resource::BindGroup, texture::Image},
    utils::EntityHashMap,
};

#[derive(Resource, Default)]
pub struct UiBindGroups {
    pub ui_tilemap_uniforms: EntityHashMap<Entity, BindGroup>,
    pub colored_texture: EntityHashMap<Handle<Image>, BindGroup>,
}
