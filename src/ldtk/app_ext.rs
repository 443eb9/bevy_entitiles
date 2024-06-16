use bevy::{
    app::App,
    ecs::{bundle::Bundle, component::Component},
};

use crate::ldtk::traits::{
    LdtkEntity, LdtkEntityRegistry, LdtkEntityTag, LdtkEntityTagRegistry, PhantomLdtkEntity,
    PhantomLdtkEntityTag,
};

pub trait LdtkApp {
    fn register_ldtk_entity<T: LdtkEntity + Bundle>(&mut self, ident: &str) -> &mut App;
    fn register_ldtk_entity_tag<T: LdtkEntityTag + Component>(&mut self, tag: &str) -> &mut App;
}

impl LdtkApp for App {
    fn register_ldtk_entity<T: LdtkEntity + Bundle>(&mut self, ident: &str) -> &mut App {
        match self
            .world_mut()
            .get_non_send_resource_mut::<LdtkEntityRegistry>()
        {
            Some(mut mapper) => {
                mapper.insert(ident.to_string(), Box::new(PhantomLdtkEntity::<T>::new()));
            }
            None => {
                self.world_mut()
                    .insert_non_send_resource(LdtkEntityRegistry::default());
                self.register_ldtk_entity::<T>(ident);
            }
        }

        self
    }

    fn register_ldtk_entity_tag<T: LdtkEntityTag + Component>(&mut self, tag: &str) -> &mut App {
        match self
            .world_mut()
            .get_non_send_resource_mut::<LdtkEntityTagRegistry>()
        {
            Some(mut mapper) => {
                mapper.insert(tag.to_string(), Box::new(PhantomLdtkEntityTag::<T>::new()));
            }
            None => {
                self.world_mut()
                    .insert_non_send_resource(LdtkEntityTagRegistry::default());
                self.register_ldtk_entity_tag::<T>(tag);
            }
        }

        self
    }
}
