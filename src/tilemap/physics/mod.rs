use bevy::{
    app::Plugin,
    ecs::{component::Component, entity::Entity, event::Event, system::Query},
    math::UVec2,
    reflect::Reflect,
};

#[cfg(feature = "physics_rapier")]
pub mod rapier;
#[cfg(feature = "physics_xpbd")]
pub mod xpbd;

pub struct EntiTilesPhysicsTilemapPlugin;

impl Plugin for EntiTilesPhysicsTilemapPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<TileCollision>();

        app.register_type::<TileCollision>()
            .register_type::<CollisionData>()
            .register_type::<PhysicsTile>();

        #[cfg(feature = "physics_rapier")]
        app.add_plugins(crate::tilemap::physics::rapier::PhysicsRapierTilemapPlugin);
        #[cfg(feature = "physics_xpbd")]
        app.add_plugins(crate::tilemap::physics::xpbd::PhysicsXpbdTilemapPlugin);
    }
}

#[derive(Component, Reflect)]
pub struct PhysicsTile {
    pub index: UVec2,
}

#[derive(Event, Debug, Reflect)]
pub enum TileCollision {
    Started(CollisionData),
    Stopped(CollisionData),
}

#[derive(Debug, Reflect, Clone)]
pub struct CollisionData {
    pub tile_index: UVec2,
    pub collider_entity: Entity,
}

fn get_collision(e1: Entity, e2: Entity, query: &Query<&PhysicsTile>) -> Option<CollisionData> {
    let (e_tile, e_other, tile) = {
        if let Ok(t) = query.get(e1) {
            (e1, e2, t)
        } else if let Ok(t) = query.get(e2) {
            (e2, e1, t)
        } else {
            return None;
        }
    };

    Some(CollisionData {
        tile_index: tile.index,
        collider_entity: e_other,
    })
}
