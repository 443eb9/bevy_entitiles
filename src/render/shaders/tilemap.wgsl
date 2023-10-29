#ifdef SQUARE
    #import "square.wgsl" get_mesh
#endif

fn tilemap_vertex(input: VertexInput) -> VertexOutput {
    let output: VertexOutput;
    output.position = vec4<f32>(10.0, 10.0, 10.0, 10.0);
    return output;
}

fn tilemap_fragment() -> @location(0) vec4<f32>{
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}