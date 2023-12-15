// MAX_ANIM_COUNT 64
// MAX_ANIM_SEQ_LENGTH 16

#import bevy_ui::ui_vertex_output::UiVertexOutput

struct TileAnimation {
    // MAX_ANIM_SEQ_LENGTH / 4
    // because array stride must be a multiple of 16 bytes
    seq: array<vec4<u32>, 4>,
    length: u32,
    fps: f32,
}

struct UiTile {
    color: vec4<f32>,
    atlas_size: vec2<f32>,
    atlas_count: vec2<u32>,
    texture_index: u32,
    flip: u32,
    time: f32,
    anim: TileAnimation,
}

@group(1) @binding(0)
var texture: texture_2d<f32>;

@group(1) @binding(1)
var texture_sampler: sampler;

@group(1) @binding(2)
var<uniform> tile: UiTile;

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    var index = tile.texture_index;

    if tile.anim.length > 0u {
        var frame = u32(tile.time * tile.anim.fps) % tile.anim.length;
        index = tile.anim.seq[frame / 4u][frame % 4u];
    }

    var mesh_uv = in.uv;
    if (tile.flip & 1u) != 0u {
        mesh_uv.x = 1.0 - mesh_uv.x;
    }
    if (tile.flip & 2u) != 0u {
        mesh_uv.y = 1.0 - mesh_uv.y;
    }

    let grid_index = vec2<u32>(index % tile.atlas_count.x, index / tile.atlas_count.x);
    let min = vec2<f32>(grid_index) * tile.atlas_size;
    let uv = min + mesh_uv * tile.atlas_size;
    let tex_size = vec2<f32>(tile.atlas_count) * tile.atlas_size;

    return textureSample(texture, texture_sampler, uv / tex_size) * pow(tile.color, vec4<f32>(2.2));
}
