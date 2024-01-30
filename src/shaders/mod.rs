use bevy::{
    app::Plugin,
    asset::{load_internal_asset, Handle},
    render::render_resource::Shader,
};

pub struct EntiTilesShaderPlugin;

pub const MATH_SHADER: Handle<Shader> = Handle::weak_from_u128(68354165413586415);

impl Plugin for EntiTilesShaderPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        load_internal_asset!(
            app,
            MATH_SHADER,
            "math.wgsl",
            Shader::from_wgsl
        );
    }
}
