// MAX_LAYER_COUNT 4
// MAX_TILESET_COUNT 4
// MAX_ATLAS_COUNT 512
// MAX_ANIM_COUNT 32
// MAX_ANIM_SEQ_LENGTH 32

#define_import_path bevy_entitiles::common

#ifdef PURE_COLOR
struct VertexInput {
    @builtin(vertex_index) v_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) index: vec4<f32>,
    @location(2) color: vec4<f32>,
}

#else // PURE_COLOR

struct VertexInput {
    @builtin(vertex_index) v_index: u32,
    @location(0) position: vec3<f32>,
    // when the third component of index is negative,
    // it means this tile is a animated tile
    // so the first component of atlas_indices is the index of the animation

    // the forth component of index is:
    // 0-1: not flipped
    // 1-2: flipped horizontally
    // 2-3: flipped vertically
    // 3-4: flipped diagonally
    @location(1) index: vec4<f32>,
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
    // when the third component of index is negative,
    // it means this tile is a animated tile
    @location(5) is_animated: f32,
#endif
}

struct TileAnimation {
    // MAX_ANIM_SEQ_LENGTH / 4
    // because array stride must be a multiple of 16 bytes
    seq: array<vec4<u32>, 8>,
    length: u32,
    fps: f32,
}

// TODO move the anim seqs and atlas uvs to a separate buffer
//      and use runtime sized arrays
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
    atlas_uvs: array<vec4<f32>, 512>,
    // MAX_ANIM_COUNT MAX_ANIM_SEQ_LENGTH
    anim_seqs: array<TileAnimation, 32>,
    time: f32,
}

@group(1) @binding(0)
var<uniform> tilemap: Tilemap;

#ifndef PURE_COLOR
@group(2) @binding(0)
var color_texture: texture_2d<f32>;

@group(2) @binding(1)
var color_texture_sampler: sampler;
#endif
