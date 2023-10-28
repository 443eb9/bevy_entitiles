use bevy::prelude::*;
use bevy_entitiles::EntiTilesPlugin;

fn main() {
    let mut app = App::new();
    app.add_plugins((DefaultPlugins, EntiTilesPlugin));
    bevy_mod_debugdump::print_schedule_graph(&mut app, Update);
    app.run();
}
