use bevy::reflect::DynamicStruct;

use super::json::level::EntityInstance;

pub trait LdtkEntity {
    fn spawn(data: &EntityInstance, dyn_struct: &mut DynamicStruct) -> Self;
}
