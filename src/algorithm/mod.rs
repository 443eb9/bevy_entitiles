use bevy::prelude::{Plugin, Update};

use self::{pathfinding::pathfinding, wfc::wave_function_collapse};

pub mod pathfinding;
pub mod wfc;

pub struct EntitilesAlgorithmPlugin;

impl Plugin for EntitilesAlgorithmPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Update, (pathfinding, wave_function_collapse));
    }
}
