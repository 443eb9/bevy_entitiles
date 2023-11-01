#import bevy_entitiles::common VertexInput, VertexOutput
#import bevy_sprite::mesh2d_view_bindings view

#ifdef SQUARE
    #import bevy_entitiles::square get_mesh
#endif

@vertex
fn tilemap_vertex(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.position = view.view_proj * input.position;
    return output;
}

@fragment
fn tilemap_fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1., 0., 0., 0.5);
}
