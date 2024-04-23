// The definition of TilemapVertexOutput is in src/render/shaders/common.wgsl
// Don't be afraid of reading the original shader code if you are not familiar with it!
// They are already filled with comments and easy to understand.
#import bevy_entitiles::common::TilemapVertexOutput;

@group(4) @binding(0)
var<uniform> speed_and_time: vec2<f32>;

// The fragment entry name of your shader must be tilemap_fragment
@fragment
fn tilemap_fragment(input: TilemapVertexOutput) -> @location(0) vec4<f32> {
    // Infact the tilemap has 4 layers. Here we only sample the top layer.
    // If you want to see how the original shader looks like, you can check src/render/shaders/tilemap.wgsl
    let tex_color = textureSample(bevy_entitiles::common::color_texture,
                              bevy_entitiles::common::color_texture_sampler,
                              input.uv, input.texture_indices[3]);
    let t = speed_and_time[0] * speed_and_time[1];
    let color = vec4<f32>(sin(t), cos(t), sin(2. * t), 1.);
    return color * tex_color;
}
