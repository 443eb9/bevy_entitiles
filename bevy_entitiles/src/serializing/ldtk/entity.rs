use bevy::reflect::DynamicStruct;

use super::json::level::EntityInstance;

pub trait LdtkEntity {
    fn spawn(entity_instance: EntityInstance) -> Self;
}
