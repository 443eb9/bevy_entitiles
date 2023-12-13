use bevy::{app::App, ecs::bundle::Bundle};

use super::entity::{LdtkEntity, LdtkEntityIdentMapper, LdtkEntityTypeMarker};

pub trait AppExt {
    fn register_ldtk_entity<T: LdtkEntity + Bundle>(&mut self, ident: &str);
}

impl AppExt for App {
    fn register_ldtk_entity<T: LdtkEntity + Bundle>(&mut self, ident: &str) {
        match self
            .world
            .get_non_send_resource_mut::<LdtkEntityIdentMapper>()
        {
            Some(mut mapper) => {
                mapper.insert(
                    ident.to_string(),
                    Box::new(LdtkEntityTypeMarker::<T>::new()),
                );
            }
            None => {
                self.world
                    .insert_non_send_resource(LdtkEntityIdentMapper::default());
                self.register_ldtk_entity::<T>(ident);
            }
        }
    }
}
