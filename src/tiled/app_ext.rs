use bevy::{app::App, ecs::bundle::Bundle};

use crate::tiled::traits::{
    PhantomTiledCustomTile, PhantomTiledObject, TiledCustomTile, TiledCustomTileRegistry,
    TiledObject, TiledObjectRegistry,
};

pub trait TiledApp {
    fn register_tiled_object<T: TiledObject + Bundle>(&mut self, ident: &str) -> &mut Self;
    fn register_tiled_custom_tile<T: TiledCustomTile + Bundle>(&mut self, ident: &str)
        -> &mut Self;
}

impl TiledApp for App {
    fn register_tiled_object<T: TiledObject + Bundle>(&mut self, ident: &str) -> &mut Self {
        match self
            .world
            .get_non_send_resource_mut::<TiledObjectRegistry>()
        {
            Some(mut registry) => {
                registry.insert(ident.to_string(), Box::new(PhantomTiledObject::<T>::new()));
                self
            }
            None => {
                self.world
                    .insert_non_send_resource(TiledObjectRegistry::default());
                self.register_tiled_object::<T>(ident)
            }
        }
    }

    fn register_tiled_custom_tile<T: TiledCustomTile + Bundle>(
        &mut self,
        ident: &str,
    ) -> &mut Self {
        match self
            .world
            .get_non_send_resource_mut::<TiledCustomTileRegistry>()
        {
            Some(mut registry) => {
                registry.insert(
                    ident.to_string(),
                    Box::new(PhantomTiledCustomTile::<T>::new()),
                );
                self
            }
            None => {
                self.world
                    .insert_non_send_resource(TiledCustomTileRegistry::default());
                self.register_tiled_custom_tile::<T>(ident)
            }
        }
    }
}
