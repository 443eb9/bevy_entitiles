// MAX_LAYER_COUNT 4
// MAX_TILESET_COUNT 4
// MAX_ANIM_COUNT 64
// MAX_ANIM_SEQ_LENGTH 16

#define_import_path bevy_entitiles::common

struct VertexInput {
    @builtin(vertex_index) v_index: u32,
    @location(0) position: vec3<f32>,
    // when the third component of index is 1,
    // it means this tile is a animated tile
    // so the first component of texture_indices is the index of the animation
    // the fourth component is flipping
    @location(1) index: vec4<u32>,
    @location(2) color: vec4<f32>,
#ifndef PURE_COLOR
    @location(3) texture_indices: vec4<i32>,
#endif
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
#ifndef PURE_COLOR
    // MAX_LAYER_COUNT
    @location(1) uv: vec2<f32>,
    @location(2) texture_indices: vec4<i32>,
    @location(3) is_animated: u32,
#endif
}

struct TileAnimation {
    // MAX_ANIM_SEQ_LENGTH / 4
    // because array stride must be a multiple of 16 bytes
    seq: array<vec4<u32>, 4>,
    length: u32,
    fps: f32,
}

struct Tilemap {
    translation: vec2<f32>,
    tile_render_size: vec2<f32>,
    ext_dir: vec2<f32>,
    tile_slot_size: vec2<f32>,
    pivot: vec2<f32>,
    // MAX_ANIM_COUNT MAX_ANIM_SEQ_LENGTH
    anim_seqs: array<TileAnimation, 64>,
    layer_opacities: vec4<f32>,
    // this value will only be meaningful when the tilemap is hexagonal!
    hex_legs: f32,
    time: f32,
}

@group(1) @binding(0)
var<uniform> tilemap: Tilemap;

#ifndef PURE_COLOR
// @group(2) @binding(0)
// var<storage> anim_seqs: array<TileAnimation>;

@group(2) @binding(0)
var color_texture: texture_2d_array<f32>;

@group(2) @binding(1)
var color_texture_sampler: sampler;
#endif
