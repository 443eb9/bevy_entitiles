use bevy::{app::App, ecs::bundle::Bundle};

use super::entities::{LdtkEntity, LdtkEntityRegistry, PhantomLdtkEntity};

pub trait AppExt {
    fn register_ldtk_entity<T: LdtkEntity + Bundle>(&mut self, ident: &str) -> &mut App;
}

impl AppExt for App {
    fn register_ldtk_entity<T: LdtkEntity + Bundle>(&mut self, ident: &str) -> &mut App {
        match self
            .world
            .get_non_send_resource_mut::<LdtkEntityRegistry>()
        {
            Some(mut mapper) => {
                mapper.insert(
                    ident.to_string(),
                    Box::new(PhantomLdtkEntity::<T>::new()),
                );
            }
            None => {
                self.world
                    .insert_non_send_resource(LdtkEntityRegistry::default());
                self.register_ldtk_entity::<T>(ident);
            }
        }

        self
    }
}
