#define_import_path bevy_entitiles::common

struct TilemapVertexInput {
    @builtin(vertex_index) v_index: u32,
    @location(0) position: vec3f,
    // When the third and forth component of index are not -1,
    // it means this tile is a animated tile.
    // So the zw components are the start index and the length of the animation sequence.
    @location(1) index: vec4i,
    @location(2) tint: vec4f,
#ifndef PURE_COLOR
    @location(3) texture_indices: vec4i,
    @location(4) flip: vec4u,
#endif
}

struct TilemapVertexOutput {
    @builtin(position) position: vec4f,
    @location(0) tint: vec4f,
#ifndef PURE_COLOR
    @location(1) uv: vec2<f32>,
    @location(2) flip: vec4u,
    @location(3) texture_indices: vec4i,
    // Indicates whether the tile is animated.
    @location(4) anim_flag: i32,
#endif
}

struct Tilemap {
    translation: vec2f,
    rot_mat: mat2x2f,
    // uv_rot: u32,
    tile_render_size: vec2f,
    slot_size: vec2f,
    pivot: vec2f,
    layer_opacities: vec4f,
    axis_dir: vec2f,
    // this value will only be meaningful when the tilemap is hexagonal!
    hex_legs: f32,
    time: f32,
// #ifdef ATLAS
//     // texture size in tiles
//     texture_tiled_size: vec2<i32>,
//     tile_uv_size: vec2<f32>,
// #endif
}

struct StandardTilemapUniform {
    color: vec4f,
}

@group(1) @binding(0)
var<uniform> tilemap: Tilemap;

@group(2) @binding(0)
var<uniform> material: StandardTilemapUniform;

#ifndef PURE_COLOR
@group(3) @binding(0)
var color_texture: texture_2d_array<f32>;

@group(3) @binding(1)
var color_texture_sampler: sampler;

@group(4) @binding(0)
var<storage> anim_seqs: array<i32>;

struct TilemapTextureDescriptor {
    tile_count: vec2u,
    tile_uv_size: vec2f,
    uv_rot: u32,
}

@group(4) @binding(1)
var<storage> texture_descs: array<TilemapTextureDescriptor>;
#endif
