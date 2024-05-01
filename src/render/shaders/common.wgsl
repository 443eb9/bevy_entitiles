#define_import_path bevy_entitiles::common

struct TilemapVertexInput {
    @builtin(vertex_index) v_index: u32,
    @location(0) position: vec3<f32>,
    // When the third and forth component of index are not -1,
    // it means this tile is a animated tile.
    // So the zw components are the start index and the length of the animation sequence.
    @location(1) index: vec4<i32>,
    @location(2) tint: vec4<f32>,
#ifndef PURE_COLOR
    @location(3) texture_indices: vec4<i32>,
    @location(4) flip: vec4<u32>,
#endif
}

struct TilemapVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tint: vec4<f32>,
#ifndef PURE_COLOR
    @location(1) uv: vec2<f32>,
    @location(2) flip: vec4<u32>,
    @location(3) texture_indices: vec4<i32>,
    // Indicates whether the tile is animated.
    @location(4) anim_flag: i32,
#endif
}

struct Tilemap {
    translation: vec2<f32>,
    rot_mat: mat2x2<f32>,
    uv_rot: u32,
    tile_render_size: vec2<f32>,
    slot_size: vec2<f32>,
    pivot: vec2<f32>,
    layer_opacities: vec4<f32>,
    axis_dir: vec2<f32>,
    // this value will only be meaningful when the tilemap is hexagonal!
    hex_legs: f32,
    time: f32,
#ifdef ATLAS
    // texture size in tiles
    texture_tiled_size: vec2<i32>,
    tile_uv_size: vec2<f32>,
#endif
}

@group(1) @binding(0)
var<uniform> tilemap: Tilemap;

#ifndef PURE_COLOR
#ifdef ATLAS
@group(3) @binding(0)
var color_texture: texture_2d<f32>;
#else
@group(3) @binding(0)
var color_texture: texture_2d_array<f32>;
#endif

@group(3) @binding(1)
var color_texture_sampler: sampler;

@group(4) @binding(0)
var<storage> anim_seqs: array<i32>;
#endif
