#import bevy_entitiles::common VertexInput, VertexOutput, texture, texture_sampler, tilemap
#import bevy_sprite::mesh2d_view_bindings view

#ifdef SQUARE
    #import bevy_entitiles::square get_mesh
#endif

#ifdef ISO_DIAMOND
    #import bevy_entitiles::iso_diamond get_mesh
#endif

@vertex
fn tilemap_vertex(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    let mesh_output = get_mesh(input);
    var position_model = vec4<f32>(
        mesh_output.position + mesh_output.translation * tilemap.tile_render_size,
        0., 1.
    );
    var position_world = tilemap.transform * position_model;
    position_model.z = input.grid_index.y;

    output.position = view.view_proj * position_world;
    output.texture_index = input.texture_index;
    output.color = input.color;

    var uv = array<vec2<f32>, 4>(
        vec2<f32>(0., 1.),
        vec2<f32>(0., 0.),
        vec2<f32>(1., 0.),
        vec2<f32>(1., 1.),
    );

    output.uv = uv[input.v_index % 4u];
#ifdef FLIP_H
    output.uv.x = 1. - output.uv.x;
#endif
#ifdef FLIP_V
    output.uv.y = 1. - output.uv.y;
#endif
    return output;
}

@fragment
fn tilemap_fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    // return vec4<f32>(input.uv, 0., 1.);
#ifdef PURE_COLOR
    let color = input.color;
#else
    let color = textureSample(texture, texture_sampler, input.uv, input.texture_index);
#endif
    return color;
}
