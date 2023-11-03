#define_import_path bevy_entitiles::square

#import bevy_entitiles::common VertexInput, MeshOutput, tilemap

fn get_mesh(input: VertexInput) -> MeshOutput {
    var output: MeshOutput;

    output.position = input.grid_index * tilemap.tile_render_size;
    var translations = array<vec2<f32>, 4>(
        vec2<f32>(0., 0.),
        vec2<f32>(0., 1.),
        vec2<f32>(1., 1.),
        vec2<f32>(1., 0.),
    );
    
    output.translation = translations[input.v_index % 4u];
    return output;
}
