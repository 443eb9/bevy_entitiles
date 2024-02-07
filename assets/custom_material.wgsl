#import bevy_entitiles::common::TilemapVertexOutput;

@group(2) @binding(0)
var<uniform> speed_and_time: vec2<f32>;

@fragment
fn tilemap_fragment(input: TilemapVertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(bevy_entitiles::common::color_texture,
                              bevy_entitiles::common::color_texture_sampler,
                              input.uv, input.texture_indices[3]);
    let t = speed_and_time[0] * speed_and_time[1];
    let color = vec4<f32>(sin(t), cos(t), 1., 1.);
    return color * tex_color;
}
