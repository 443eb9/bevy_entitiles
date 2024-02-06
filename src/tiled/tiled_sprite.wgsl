#import bevy_sprite::{mesh2d_vertex_output::TilemapVertexOutput}
#import bevy_entitiles::math::Aabb2d

struct SpriteUniform {
    atlas: Aabb2d,
    tint: vec4<f32>,
}

@group(1) @binding(0)
var texture: texture_2d<f32>;

@group(1) @binding(1)
var texture_sampler: sampler;

@group(1) @binding(2)
var<uniform> data: SpriteUniform;

@fragment
fn fragment(in: TilemapVertexOutput) -> @location(0) vec4<f32> {
    return textureSample(texture, texture_sampler, in.uv * (data.atlas.max - data.atlas.min) + data.atlas.min)
           * vec4<f32>(pow(data.tint.rgb, vec3<f32>(2.2)), data.tint.a);
}
