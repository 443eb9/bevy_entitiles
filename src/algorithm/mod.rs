use bevy::prelude::{Plugin, Update};

use self::{
    pathfinding::{Path, PathTilemaps},
    wfc::{WfcData, WfcElement, WfcHistory, WfcSource},
};

pub mod pathfinding;
pub mod wfc;

pub struct EntiTilesAlgorithmPlugin;

impl Plugin for EntiTilesAlgorithmPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_type::<Path>();

        app.register_type::<WfcElement>()
            .register_type::<WfcHistory>()
            .register_type::<WfcData>()
            .register_type::<WfcSource>();

        app.init_resource::<PathTilemaps>();

        app.add_systems(
            Update,
            (
                pathfinding::pathfinding_scheduler,
                pathfinding::path_assigner,
                wfc::wave_function_collapse,
                wfc::wfc_data_assigner,
                wfc::wfc_applier,
                #[cfg(feature = "ldtk")]
                wfc::ldtk_wfc_helper,
            ),
        );
    }
}
