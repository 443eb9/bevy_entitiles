struct VertexInput {
    @builtin(vertex_index) v_index: u32,
    @location(0) position: vec4<f32>,
    @location(1) uv: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

struct MeshOutput {
    position_ws: vec4<f32>,
    uv: vec2<f32>,
}
