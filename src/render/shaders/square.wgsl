#define_import_path bevy_entitiles::square

#import bevy_entitiles::common VertexInput, MeshOutput, tilemap

fn get_mesh(input: VertexInput) -> MeshOutput {
    var output: MeshOutput;

    let p_bl = input.grid_index * tilemap.tile_render_size;
    var mesh = array<vec2<f32>, 4>(
        p_bl,
        p_bl + vec2<f32>(0., tilemap.tile_render_size.y),
        p_bl + vec2<f32>(tilemap.tile_render_size.x, tilemap.tile_render_size.y),
        p_bl + vec2<f32>(tilemap.tile_render_size.x, 0.),
    );

    var uv = array<vec2<f32>, 4>(
        vec2<f32>(0., 0.),
        vec2<f32>(0., 1.),
        vec2<f32>(1., 1.),
        vec2<f32>(1., 0.),
    );
    
    let index = input.v_index % 4u;
    output.position = tilemap.transform * vec4<f32>(mesh[index], 0., 1.);
    output.uv = uv[index];
    return output;
}
