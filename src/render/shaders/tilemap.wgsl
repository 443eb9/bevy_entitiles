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
    output.uv = mesh_output.uv;
    output.color = input.color;
    return output;
}

@fragment
fn tilemap_fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    // return vec4<f32>(input.uv, 0., 1.);
    let color = textureSample(texture, texture_sampler, input.uv, input.texture_index) * input.color;
    return color;
}
