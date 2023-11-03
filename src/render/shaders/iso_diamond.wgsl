#define_import_path bevy_entitiles::iso_diamond

#import bevy_entitiles::common VertexInput, MeshOutput, tilemap

fn get_mesh(input: VertexInput) -> MeshOutput {
    var output: MeshOutput;
    output.position = vec2<f32>(
        max(input.grid_index.x, input.grid_index.y) / 2. * tilemap.tile_render_size.x,
        (input.grid_index.x + input.grid_index.y) / 2. * tilemap.tile_render_size.y
    );
    if input.grid_index.y > input.grid_index.x {
        output.position.x *= -1.;
    }

    var translations = array<vec2<f32>, 4>(
        vec2<f32>(-0.5, 0.),
        vec2<f32>(-0.5, 0.5),
        vec2<f32>(0.5, 0.),
        vec2<f32>(0.5, -0.5),
    );

    output.translation = translations[input.v_index % 4u];
    return output;
}