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
var<uniform> atlas_rect: AtlasRect;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(texture, texture_sampler,
                         in.uv * (atlas_rect.max - atlas_rect.min) + atlas_rect.min);
    // return vec4<f32>(mix(textureSample(texture, texture_sampler,
    //                      in.uv * (atlas_rect.max - atlas_rect.min) + atlas_rect.min).rgb, vec3<f32>(1., 1., 1.), 0.2), 1.);
    // return vec4<f32>(1.0);
}
