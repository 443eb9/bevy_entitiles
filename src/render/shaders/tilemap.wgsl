#import bevy_entitiles::common VertexInput, VertexOutput

#ifdef SQUARE
    #import bevy_entitiles::square get_mesh
#endif

@vertex
fn tilemap_vertex(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.position = vec4<f32>(10.0, 10.0, 10.0, 10.0);
    return output;
}

@fragment
fn tilemap_fragment(input: VertexOutput) -> @location(0) vec4<f32>{
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
