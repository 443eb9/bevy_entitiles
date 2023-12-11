// MAX_LAYER_COUNT 4
// MAX_TILESET_COUNT 4
// MAX_ATLAS_COUNT 1024

#define_import_path bevy_entitiles::common

#ifdef PURE_COLOR
struct VertexInput {
    @builtin(vertex_index) v_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) index: vec2<f32>,
    @location(2) color: vec4<f32>,
}

#else // PURE_COLOR

struct VertexInput {
    @builtin(vertex_index) v_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) index: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) atlas_indices: vec4<i32>,
}

#endif // PURE_COLOR

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
#ifndef PURE_COLOR
    // MAX_LAYER_COUNT
    @location(1) uv_a: vec2<f32>,
    @location(2) uv_b: vec2<f32>,
    @location(3) uv_c: vec2<f32>,
    @location(4) uv_d: vec2<f32>,
#endif
}

struct Tilemap {
    translation: vec2<f32>,
    // when the tilemap is a uniform one,
    // the tile_render_size below is used
    tile_render_size: vec2<f32>,
    tile_render_scale: vec2<f32>,
    tile_slot_size: vec2<f32>,
    pivot: vec2<f32>,
    texture_size: vec2<f32>,
    // MAX_ATLAS_COUNT
    atlas_uvs: array<vec4<f32>, 1024>,
}

@group(1) @binding(0)
var<uniform> tilemap: Tilemap;

#ifndef PURE_COLOR
@group(2) @binding(0)
var color_texture: texture_2d<f32>;

@group(2) @binding(1)
var color_texture_sampler: sampler;
#endif
