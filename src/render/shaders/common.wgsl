#define_import_path bevy_entitiles::common

struct VertexInput {
    @builtin(vertex_index) v_index: u32,
    @location(0) position: vec4<f32>,
    @location(1) texture_index: u32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

struct MeshOutput {
    position_ws: vec4<f32>,
    uv: vec2<f32>,
}
