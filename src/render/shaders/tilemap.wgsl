#import bevy_entitiles::common VertexInput, VertexOutput, texture, texture_sampler
#import bevy_sprite::mesh2d_view_bindings view

#ifdef SQUARE
    #import bevy_entitiles::square get_mesh
#endif

@vertex
fn tilemap_vertex(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    let mesh_output = get_mesh(input);
    output.position = view.view_proj * mesh_output.position;
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
    let color = textureSample(texture, texture_sampler, input.uv, input.texture_index);
    return color;
}
