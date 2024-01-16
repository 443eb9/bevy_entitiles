use bevy::{
    app::App,
    ecs::{bundle::Bundle, component::Component},
};

use super::traits::{
    LdtkEntity, LdtkEntityRegistry, LdtkEntityTag, LdtkEnum, PhantomLdtkEntity,
    PhantomLdtkEntityTag,
};

pub trait AppExt {
    fn register_ldtk_entity<T: LdtkEntity + Bundle>(&mut self, ident: &str) -> &mut App;
    fn register_ldtk_entity_tag<T: LdtkEnum + Component>(&mut self) -> &mut App;
}

impl AppExt for App {
    fn register_ldtk_entity<T: LdtkEntity + Bundle>(&mut self, ident: &str) -> &mut App {
        match self.world.get_non_send_resource_mut::<LdtkEntityRegistry>() {
            Some(mut mapper) => {
                mapper.insert(ident.to_string(), Box::new(PhantomLdtkEntity::<T>::new()));
            }
            None => {
                self.world
                    .insert_non_send_resource(LdtkEntityRegistry::default());
                self.register_ldtk_entity::<T>(ident);
            }
        }

        self
    }

    fn register_ldtk_entity_tag<T: LdtkEnum + Component>(&mut self) -> &mut App {
        match self.world.get_non_send_resource_mut::<LdtkEntityTag>() {
            Some(mut tag) => {
                tag.0 = Box::new(PhantomLdtkEntityTag::<T>::new());
            }
            None => {
                self.world.insert_non_send_resource(LdtkEntityTag(Box::new(
                    PhantomLdtkEntityTag::<T>::new(),
                )));
            }
        }

        self
    }
}
