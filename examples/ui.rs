use bevy::{app::App, DefaultPlugins};
use bevy_entitiles::ui::EntiTilesUiPlugin;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EntiTilesUiPlugin))
        .run();
}
