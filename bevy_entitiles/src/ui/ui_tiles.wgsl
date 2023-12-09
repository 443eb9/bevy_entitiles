#import bevy_ui::ui_vertex_output::UiVertexOutput

struct TileUV {
    min: vec2<f32>,
    max: vec2<f32>,
}

struct UiTile {
    @location(0) uv: TileUV,
    @location(1) color: vec4<f32>,
    @location(2) texture_size: vec2<f32>,
}

@group(1) @binding(0)
var texture: texture_2d<f32>;

@group(1) @binding(1)
var texture_sampler: sampler;

@group(1) @binding(2)
var<uniform> tile: UiTile;

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let area = tile.uv.max - tile.uv.min;
    let uv = in.uv * area + tile.uv.min;
    return textureSample(texture, texture_sampler, uv / tile.texture_size) * tile.color;
}
