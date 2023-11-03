#define_import_path bevy_entitiles::common

struct VertexInput {
    @builtin(vertex_index) v_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) grid_index: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) texture_index: u32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) texture_index: u32,
}

struct Tilemap {
    transform: mat4x4<f32>,
    tile_render_size: vec2<f32>,
}

@group(1) @binding(0)
var<uniform> tilemap: Tilemap;

#ifndef PURE_COLOR
@group(2) @binding(0)
var texture: texture_2d_array<f32>;

@group(2) @binding(1)
var texture_sampler: sampler;
#endif
