#import bevy_sprite::{mesh2d_vertex_output::VertexOutput}

struct AtlasRect {
    min: vec2<f32>,
    max: vec2<f32>,
}

@group(1) @binding(0)
var texture: texture_2d<f32>;

@group(1) @binding(1)
var texture_sampler: sampler;

@group(1) @binding(2)
var<uniform> atlas_rects: AtlasRect;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(texture, texture_sampler, in.uv * (atlas_rects.max - atlas_rects.min) + atlas_rects.min);
}
